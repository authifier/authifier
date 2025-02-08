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
    Discoverable,
    Manual {
        authorization: String,
        token: String,
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
pub struct IdProvider {
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

impl Borrow<str> for IdProvider {
    fn borrow(&self) -> &str {
        &*self.id
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

#[derive(Default, Clone)]
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
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl Deref for SSO {
    type Target = HashSet<IdProvider>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
