use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::config::{Claim, Credentials, Endpoints};

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
