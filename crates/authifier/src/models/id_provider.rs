use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::config::{Claim, Credentials, Endpoints, IdProvider as IdProviderConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdProvider {
    pub id: String,

    pub issuer: reqwest::Url,
    pub name: Option<String>,
    pub icon: Option<reqwest::Url>,

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

impl TryFrom<IdProviderConfig> for IdProvider {
    type Error = <reqwest::Url as std::str::FromStr>::Err;

    fn try_from(config: IdProviderConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            id: config.id,
            issuer: config.issuer.parse()?,
            name: config.name,
            icon: config.icon.as_deref().map(str::parse).transpose()?,
            scopes: config.scopes,
            endpoints: config.endpoints,
            credentials: config.credentials,
            claims: config.claims,
            code_challenge: config.code_challenge,
        })
    }
}
