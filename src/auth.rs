use std::collections::HashMap;

use crate::email;
use crate::options::{Templates, SMTP};
use crate::util::normalise_email;

use super::options::Options;
use super::util::{Error, Result};

use argon2::{self, Config};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::Collection;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

pub struct Auth {
    pub collection: Collection,
    pub options: Options,
}

lazy_static! {
    static ref ARGON_CONFIG: Config<'static> = Config::default();
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Session {
    #[validate(length(min = 26, max = 26))]
    pub id: Option<String>,
    #[validate(length(min = 26, max = 26))]
    pub user_id: String,
    #[validate(length(min = 64, max = 64))]
    pub session_token: String,
}

#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
    success: bool,
}

impl Auth {
    pub fn new(collection: Collection, options: Options) -> Auth {
        Auth {
            collection,
            options,
        }
    }

    pub async fn verify_session(&self, mut session: Session) -> Result<Session> {
        let doc = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "sessions.$": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidSession)?;

        session.id = Some(
            doc.get_array("sessions")
                .map_err(|_| Error::DatabaseError {
                    operation: "get_array(sessions)",
                    with: "account",
                })?
                .iter()
                .next()
                .ok_or(Error::DatabaseError {
                    operation: "next()",
                    with: "array(sessions)",
                })?
                .as_document()
                .ok_or(Error::DatabaseError {
                    operation: "as_document()",
                    with: "array(sessions)",
                })?
                .get_str("id")
                .map_err(|_| Error::DatabaseError {
                    operation: "get_str(id)",
                    with: "array(sessions)",
                })?
                .to_string(),
        );

        Ok(session)
    }

    pub async fn fetch_password(&self, session: &Session) -> Result<String> {
        let user = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "password": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidCredentials)?;

        Ok(user
            .get_str("password")
            .map_err(|_| Error::DatabaseError {
                operation: "get_str(password)",
                with: "account",
            })?
            .to_string())
    }

    pub async fn verify_password(&self, session: &Session, password: String) -> Result<()> {
        let hash = self.fetch_password(&session).await?;

        if argon2::verify_encoded(&hash, password.as_bytes()).map_err(|_| Error::InternalError)? {
            Ok(())
        } else {
            Err(Error::InvalidCredentials)
        }
    }

    pub async fn check_email_is_use(&self, email: String) -> Result<String> {
        let normalised = normalise_email(email);

        if self
            .collection
            .find_one(
                doc! {
                    "email_normalised": &normalised
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .is_some()
        {
            Err(Error::EmailInUse)
        } else {
            Ok(normalised)
        }
    }

    pub fn email_send_verification(
        &self,
        smtp: &SMTP,
        templates: &Templates,
        email: &String,
        code: &String,
    ) -> Result<()> {
        let url = format!("{}/verify/{}", self.options.base_url, code);
        let email =
            templates
                .verify_email
                .generate_email(&smtp.from, email, json!({ "url": url }))?;
        email::send(&smtp, email)
    }

    pub fn email_send_password_reset(
        &self,
        smtp: &SMTP,
        templates: &Templates,
        redirect: &Option<String>,
        email: &String,
        code: &String,
    ) -> Result<()> {
        let url = format!(
            "{}/{}",
            redirect
                .clone()
                .unwrap_or_else(|| format!("{}/reset", self.options.base_url)),
            code
        );

        let email =
            templates
                .reset_password
                .generate_email(&smtp.from, email, json!({ "url": url }))?;

        email::send(&smtp, email)
    }

    pub async fn verify_captcha(&self, user_token: &Option<String>) -> Result<()> {
        if let Some(key) = &self.options.hcaptcha_secret {
            if let Some(token) = user_token {
                let mut map = HashMap::new();
                map.insert("secret", key.clone());
                map.insert("response", token.to_string());

                let client = reqwest::Client::new();
                if let Ok(response) = client
                    .post("https://hcaptcha.com/siteverify")
                    .form(&map)
                    .send()
                    .await
                {
                    let result: CaptchaResponse =
                        response.json().await.map_err(|_| Error::CaptchaFailed)?;

                    if result.success {
                        Ok(())
                    } else {
                        Err(Error::CaptchaFailed)
                    }
                } else {
                    Err(Error::CaptchaFailed)
                }
            } else {
                Err(Error::CaptchaFailed)
            }
        } else {
            Ok(())
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for Session {
    type Error = Error;

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let header_user_id = request
            .headers()
            .get("x-user-id")
            .next()
            .map(|x| x.to_string());

        let header_session_token = request
            .headers()
            .get("x-session-token")
            .next()
            .map(|x| x.to_string());

        match (
            request.managed_state::<Auth>(),
            header_user_id,
            header_session_token,
        ) {
            (Some(auth), Some(user_id), Some(session_token)) => {
                let session = Session {
                    id: None,
                    user_id,
                    session_token,
                };

                if let Ok(session) = auth.verify_session(session).await {
                    Outcome::Success(session)
                } else {
                    Outcome::Failure((Status::Forbidden, Error::InvalidSession))
                }
            }
            (None, _, _) => Outcome::Failure((Status::InternalServerError, Error::InternalError)),
            (_, _, _) => Outcome::Failure((Status::Forbidden, Error::MissingHeaders)),
        }
    }
}
