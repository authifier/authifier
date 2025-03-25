use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
};

use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Endpoints {
    Discoverable,
    Manual {
        authorization: Url,
        token: Url,
        userinfo: Url,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Credentials {
    None {
        client_id: String,
    },
    Basic {
        client_id: String,
        client_secret: String,
    },
    Post {
        client_id: String,
        client_secret: String,
    },
}

impl Credentials {
    pub fn client_id(&self) -> &str {
        match self {
            Credentials::None { client_id }
            | Credentials::Basic { client_id, .. }
            | Credentials::Post { client_id, .. } => client_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Claim {
    Id,
    Username,
    Picture,
    Email,
}

#[derive(Clone, Debug)]
pub struct IdProvider {
    pub id: String,

    pub issuer: Url,
    pub name: Option<String>,
    pub icon: Option<Url>,

    pub scopes: Vec<String>,
    pub endpoints: Endpoints,
    pub credentials: Credentials,
    pub claims: HashMap<Claim, String>,

    pub code_challenge: bool,
}

impl Borrow<str> for IdProvider {
    fn borrow(&self) -> &str {
        &self.id
    }
}

impl PartialEq for IdProvider {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for IdProvider {}

impl Hash for IdProvider {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SSO(HashSet<IdProvider>);

impl Serialize for SSO {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for SSO {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Mock {
            pub issuer: Url,
            pub name: Option<String>,
            pub icon: Option<Url>,

            pub scopes: Vec<String>,
            pub endpoints: Endpoints,
            pub credentials: Credentials,
            pub claims: HashMap<Claim, String>,

            pub code_challenge: bool,
        }

        let map: HashMap<String, Mock> =
            HashMap::deserialize(deserializer).map_err(serde::de::Error::custom)?;

        Ok(SSO(map
            .into_iter()
            .map(|(id, mock)| IdProvider {
                id,
                issuer: mock.issuer,
                name: mock.name,
                icon: mock.icon,
                scopes: mock.scopes,
                endpoints: mock.endpoints,
                credentials: mock.credentials,
                claims: mock.claims,
                code_challenge: mock.code_challenge,
            })
            .collect()))
    }
}

impl Deref for SSO {
    type Target = HashSet<IdProvider>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_sso_config() {
        let value = serde_json::json!(
            {
                "Gitlab": {
                    "issuer": "https://gitlab.com",
                    "scopes": ["openid"],

                    "endpoints": {
                        "type": "discoverable"
                    },
                    "credentials": {
                        "type": "post",
                        "client_id": "foobar",
                        "client_secret": "baz"
                    },
                    "claims": {
                        "id": "sub",
                        "email": "preferred_email"
                    },

                    "code_challenge": false,
                }
            }
        );

        let result: SSO = serde_json::from_value(value).expect("config deserializes successfully");

        assert_eq!(
            result,
            SSO([IdProvider {
                id: "Gitlab".to_owned(),

                issuer: "https://gitlab.com"
                    .parse()
                    .expect("issuer should be valid"),
                name: None,
                icon: None,

                scopes: vec!["openid".to_owned()],
                endpoints: Endpoints::Discoverable,
                credentials: Credentials::Post {
                    client_id: "foobar".to_owned(),
                    client_secret: "baz".to_owned(),
                },
                claims: [
                    (Claim::Id, "sub".to_owned()),
                    (Claim::Email, "preferred_email".to_owned())
                ]
                .into_iter()
                .collect(),

                code_challenge: false,
            }]
            .into_iter()
            .collect())
        );
    }
}
