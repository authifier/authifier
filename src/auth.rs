use super::util::{ Error, Result };

use regex::Regex;
use serde::Deserialize;
use mongodb::Collection;
use validator::{Validate, ValidationError};

pub struct Auth {
    collection: Collection
}

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[A-z0-9-]+$").unwrap();
}

#[derive(Debug, Validate, Deserialize)]
pub struct Session {
    #[validate(length(min = 26, max = 26))]
    user_id: String,
    #[validate(length(min = 64, max = 128))]
    session_token: String
}

#[derive(Debug, Validate, Deserialize)]
pub struct Create {
    #[validate(email)]
    email: String,
    #[validate(regex = "RE_USERNAME", length(min = 3, max = 32))]
    username: String,
    #[validate(length(min = 8, max = 72))]
    password: String
}

#[derive(Debug, Validate, Deserialize)]
pub struct Verify {
    #[validate(length(min = 24, max = 64))]
    pub code: String
}

#[derive(Debug, Validate, Deserialize)]
pub struct Login {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 72))]
    password: String
}

impl Auth {
    pub fn new(collection: Collection) -> Auth {
        Auth {
            collection
        }
    }

    pub async fn create_account(&self, data: Create) -> Result<String> {
        data
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

        Ok("bruh".to_string())
    }

    pub fn verify_account(&self, data: Verify) -> Result<String> {
        unimplemented!()
    }
    
    pub fn fetch_verification(&self, email: String) -> Result<String> {
        unimplemented!()
    }
    
    pub fn login(&self, data: Login) -> Result<Session> {
        unimplemented!()
    }
    
    pub fn verify_session(&self, session: Session) -> Result<bool> {
        unimplemented!()
    }
}

