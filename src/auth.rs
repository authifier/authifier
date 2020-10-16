use super::util::{ Error, Result };

use ulid::Ulid;
use regex::Regex;
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Serialize, Deserialize};
use mongodb::options::FindOneOptions;
use validator::{Validate, ValidationError};

pub struct Auth {
    collection: Collection
}

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[A-z0-9-]+$").unwrap();
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct Session {
    #[validate(length(min = 26, max = 26))]
    pub user_id: String,
    #[validate(length(min = 64, max = 128))]
    pub session_token: String
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
        
        let user_id = Ulid::new().to_string();
        self.collection.insert_one(
            doc! {
                "_id": &user_id,
                "email": data.email,
                "username": data.username,
                "password": data.password
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError)?;

        Ok(user_id)
    }

    pub async fn verify_account(&self, data: Verify) -> Result<String> {
        data
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

        unimplemented!()
    }
    
    pub async fn fetch_verification(&self, email: String) -> Result<String> {
        unimplemented!()
    }
    
    pub async fn login(&self, data: Login) -> Result<Session> {
        data
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

        let user = self.collection.find_one(
            doc! {
                "email": data.email
            },
            FindOneOptions::builder()
                .projection(doc! {
                    "_id": 1,
                    "password": 1
                })
                .build()
        )
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or(Error::UnknownUser)?;

        if &data.password != user.get_str("password")
            .map_err(|_| Error::DatabaseError)? {
            Err(Error::WrongPassword)?;
        }
        
        Ok(Session {
            user_id: user.get_str("_id").map_err(|_| Error::DatabaseError)?.to_string(),
            session_token: data.password
        })
    }
    
    pub async fn verify_session(&self, session: Session) -> Result<bool> {
        unimplemented!()
    }
}

