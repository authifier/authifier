use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Endpoints {
    /// OIDC Discovery support
    Discoverable,
    /// Manually provided endpoints
    Manual {
        authorization: String,
        token: String,
        // TODO: Optional?
        userinfo: String,
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
    #[allow(dead_code)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Provider {
    pub id: String,

    pub issuer: String,
    pub name: Option<String>,
    pub icon: Option<String>,

    pub scopes: Vec<String>,
    pub endpoints: Endpoints,
    pub credentials: Credentials,
    pub claims: HashMap<Claim, String>,

    pub code_challenge: bool,
}

impl Borrow<str> for Provider {
    fn borrow(&self) -> &str {
        &self.id
    }
}

impl PartialEq for Provider {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Provider {}

impl Hash for Provider {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

#[derive(Default, Clone)]
pub struct Providers(HashSet<Provider>);

impl Deref for Providers {
    type Target = HashSet<Provider>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Providers {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Providers {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
