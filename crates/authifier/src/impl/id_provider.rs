use std::collections::HashMap;

use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, GeneralPurpose},
    Engine,
};
use mime::{Mime, APPLICATION_JSON};
use oauth2_types::{
    oidc::{ProviderMetadata, VerifiedProviderMetadata},
    requests::{AccessTokenRequest, AccessTokenResponse, AuthorizationCodeGrant},
};
use rand::Rng;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, WWW_AUTHENTICATE},
    Url,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    config::{Credentials, Endpoints},
    models::{Callback, IdProvider},
    util::secure_random_str,
    Authifier, Error, Result,
};

static OIDC_CONFIG_PATH: &str = "/.well-known/openid-configuration";

type IdToken = HashMap<String, serde_json::Value>;

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

impl IdProvider {
    /// Create authorization URI
    pub async fn create_authorization_uri<'res>(
        &self,
        authifier: &Authifier,
        redirect_uri: &Url,
    ) -> Result<(String, Url)> {
        let state = ulid::Ulid::new().to_string();

        let nonce = match &self.endpoints {
            Endpoints::Discoverable => Some(secure_random_str(32)),
            Endpoints::Manual { .. } => None,
        };

        let (code_verifier, code_challenge) =
            self.code_challenge.then(create_code_challenge).unzip();

        let mut authorization_uri = match &self.endpoints {
            Endpoints::Discoverable => {
                let metadata = self.discover(authifier).await?;

                metadata.authorization_endpoint().to_owned()
            }
            Endpoints::Manual { authorization, .. } => authorization.parse().unwrap(),
        };

        {
            authorization_uri.query_pairs_mut().extend_pairs([
                ("client_id", self.credentials.client_id()),
                ("redirect_uri", redirect_uri.as_ref()),
                ("response_type", "code"),
                ("scope", &*self.scopes.join(" ")),
                ("state", &*state),
            ]);
        }

        if let Some(nonce) = nonce.as_deref() {
            authorization_uri
                .query_pairs_mut()
                .extend_pairs([("nonce", nonce)]);
        }

        if let Some(code_challenge) = code_challenge.as_deref() {
            authorization_uri.query_pairs_mut().extend_pairs([
                ("code_challenge", code_challenge),
                ("code_challenge_method", "S256"),
            ]);
        }

        let secret = authifier.database.find_secret().await?;

        // let builder = Cookie::build(("callback-state", secret.sign_claims(&state)))
        //     .secure(true)
        //     .http_only(true);

        // let (path, same_site, max_age) =
        //     ("/callback", SameSite::Strict, Duration::seconds(60 * 10));
        // let cookie = builder
        //     .path(path)
        //     .same_site(same_site)
        //     .max_age(max_age)
        //     .build();

        // let location = Header::new("Location", authorization_uri.to_string());

        let callback = Callback {
            id: state.clone(),
            nonce,
            code_verifier,
            ..Callback::new(self.id.clone(), redirect_uri.clone())
        };

        authifier.database.save_callback(&callback).await?;

        Ok((secret.sign_claims(&state), authorization_uri))
    }

    /// Exchange authorization code for access token
    pub async fn exchange_authorization_code(
        &self,
        authifier: &Authifier,
        code: &str,
        state: &str,
    ) -> Result<(AccessTokenResponse, Option<IdToken>)> {
        let callback = authifier.database.find_callback(state).await?;

        // validate state
        if state != callback.id {
            authifier.database.delete_callback(state).await?;

            return Err(Error::StateMismatch);
        }

        let endpoint = match &self.endpoints {
            Endpoints::Discoverable => {
                let metadata = self.discover(authifier).await?;

                metadata.token_endpoint().to_owned()
            }
            Endpoints::Manual { token, .. } => token.parse().unwrap(),
        };

        let body = AccessTokenRequest::AuthorizationCode(AuthorizationCodeGrant {
            code: code.to_owned(),
            redirect_uri: Some(callback.redirect_uri.parse().unwrap()),
            code_verifier: callback.code_verifier.clone(),
        });

        let builder = authifier.http_client.post(endpoint);

        match self.request(builder, body).await {
            Ok(res) if res.status().is_success() => {
                Ok((res.json().await.map_err(|_| Error::RequestFailed)?, None))
            }
            Ok(res) => {
                let ErrorResponse { error } = res.json().await.map_err(|_| Error::RequestFailed)?;

                Err(match &*error {
                    "invalid_request" => Error::InvalidRequest,
                    "invalid_client" => Error::InvalidClient,
                    "invalid_grant" => Error::InvalidGrant,
                    "unauthorized_client" => Error::UnauthorizedClient,
                    "unsupported_grant_type" => Error::UnsupportedGrantType,
                    "invalid_scope" => Error::InvalidScope,
                    _ => Error::RequestFailed,
                })
            }
            Err(_) => Err(Error::RequestFailed),
        }
    }

    pub async fn fetch_userinfo(
        &self,
        authifier: &Authifier,
        access_token: &str,
    ) -> Result<Option<IdToken>> {
        let Some(endpoint) = (match &self.endpoints {
            Endpoints::Discoverable => {
                let metadata = self.discover(authifier).await?;

                metadata.userinfo_endpoint.as_ref().cloned()
            }
            Endpoints::Manual { userinfo, .. } => Some(userinfo.parse().unwrap()),
        }) else {
            return Ok(None);
        };

        let builder = authifier.http_client.get(endpoint);

        let res = match self.request(builder.bearer_auth(access_token), ()).await {
            Ok(res) if res.status().is_success() => {
                let header = res.headers().get(CONTENT_TYPE);

                let Some(mime): Option<Mime> =
                    header.and_then(|h| h.to_str().ok().map(str::parse).and_then(Result::ok))
                else {
                    return Err(Error::MissingHeaders);
                };

                if mime.essence_str() != APPLICATION_JSON {
                    return Err(Error::ContentTypeMismatch);
                }

                res
            }
            Ok(res) => {
                let header = res.headers().get(WWW_AUTHENTICATE);

                let Some((_, error)) = header.and_then(|h| {
                    h.to_str().ok().and_then(|s| {
                        let it = s.trim_matches("Bearer ").split(',');

                        it.filter_map(|s| s.split_once('='))
                            .find(|(k, v)| k == "error")
                    })
                }) else {
                    return Err(Error::MissingHeaders);
                };

                return Err(match error {
                    "invalid_request" => Error::InvalidRequest,
                    "unsupported_grant_type" => Error::UnsupportedGrantType,
                    "invalid_scope" => Error::InvalidScope,
                    _ => Error::RequestFailed,
                });
            }
            Err(_) => {
                return Err(Error::RequestFailed);
            }
        };

        // TODO: Subject identifier must always be the same
        match res.json().await.map_err(|_| Error::RequestFailed)? {
            serde_json::Value::Object(userinfo) => Ok(Some(userinfo.into_iter().collect())),
            _ => {
                return Err(Error::InvalidUserinfo);
            }
        }
    }

    pub async fn request<T>(
        &self,
        builder: reqwest::RequestBuilder,
        body: T,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        T: Serialize,
    {
        /// A request with client credentials added to it.
        #[derive(Clone, Serialize)]
        struct Request<'c, T> {
            #[serde(flatten)]
            body: T,
            client_id: &'c str,
            #[serde(skip_serializing_if = "Option::is_none")]
            client_secret: Option<&'c str>,
        }

        let (client_id, client_secret) = (
            self.credentials.client_id(),
            match &self.credentials {
                Credentials::Basic { client_secret, .. }
                | Credentials::Post { client_secret, .. } => Some(&**client_secret),
                _ => None,
            },
        );

        let request = builder.form(&Request {
            body,
            client_id,
            client_secret,
        });

        let request = request.header(ACCEPT, APPLICATION_JSON.as_ref());

        if let Credentials::Basic {
            client_id,
            client_secret,
        } = &self.credentials
        {
            let (username, password): (String, String) = (
                form_urlencoded::byte_serialize(client_id.as_bytes()).collect(),
                form_urlencoded::byte_serialize(client_secret.as_bytes()).collect(),
            );

            request.basic_auth(username, Some(password)).send().await
        } else {
            request.send().await
        }
    }

    /// Fetch the provider metadata.
    async fn discover(&self, authifier: &Authifier) -> Result<VerifiedProviderMetadata> {
        let config_url = self
            .issuer
            .join(OIDC_CONFIG_PATH)
            .map_err(|_| Error::InvalidEndpoints)?;

        let request = authifier.http_client.get(config_url);
        let response = request.send().await.map_err(|_| Error::InvalidEndpoints)?;

        let metadata: ProviderMetadata =
            response.json().await.map_err(|_| Error::InvalidEndpoints)?;

        metadata
            .validate(self.issuer.as_ref())
            .map_err(|_| Error::InvalidEndpoints)
    }
}

#[inline(always)]
fn create_code_challenge() -> (String, String) {
    let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
    let mut arr = [0u8; 32];

    rand::thread_rng().fill(&mut arr);
    let code_verifier = engine.encode(arr);

    let digest = Sha256::digest(&code_verifier);
    let code_challenge = engine.encode(digest);

    (code_verifier, code_challenge)
}
