use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Serialize};

use crate::models::Secret;

impl Secret {
    /// Sign claims with secret
    pub fn sign_claims<T>(&self, claims: &T) -> String
    where
        T: Serialize,
    {
        let secret = self.expose().as_bytes();

        let (header, key) = (Header::default(), EncodingKey::from_secret(secret));

        jsonwebtoken::encode(&header, claims, &key).expect("JWT encoding should not fail")
    }

    /// Validate claims with secret
    pub fn validate_claims<T>(&self, token: &str) -> Result<T, jsonwebtoken::errors::Error>
    where
        T: DeserializeOwned,
    {
        let secret = self.expose().as_bytes();

        let (validation, key) = (Validation::default(), DecodingKey::from_secret(secret));

        jsonwebtoken::decode(token, &key, &validation).map(|token| token.claims)
    }
}
