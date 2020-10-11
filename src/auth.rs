use super::util::{ Error, Result };

use regex::Regex;
use serde::Deserialize;
use mongodb::Collection;
use validator::{Validate, ValidationError};

pub struct Session {
    user_id: String,
    session_token: String
}

pub struct Auth {
    collection: Collection
}

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[A-z0-9-]+$").unwrap();
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

impl Auth {
    pub fn new(collection: Collection) -> Auth {
        Auth {
            collection
        }
    }

    pub fn create_account(&self, data: Create) -> Result<String> {
        data
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

        Ok("bruh".to_string())
    }

    pub fn verify_account(code: &str) -> Result<String> {
        unimplemented!()
    }
    
    pub fn fetch_verification(email: String) -> Result<String> {
        unimplemented!()
    }
    
    pub fn login(email: String, password: String) -> Result<Session> {
        unimplemented!()
    }
    
    pub fn verify_session(session: &Session) -> Result<bool> {
        unimplemented!()
    }
}

