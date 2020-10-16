use super::util::{ Error, Result };

use ulid::Ulid;
use regex::Regex;
use mongodb::bson::doc;
use mongodb::Collection;
use rocket::http::Status;
use validator::{Validate};
use serde::{Serialize, Deserialize};
use mongodb::options::FindOneOptions;
use rocket::request::{self, Outcome, FromRequest, Request};

pub struct Auth {
    collection: Collection
}

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[A-z0-9-]+$").unwrap();
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
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
pub struct FetchVerification {
    #[validate(email)]
    email: String
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

    pub async fn verify_account(&self, data: Verify) -> Result<()> {
        data
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

        unimplemented!()
    }
    
    pub async fn fetch_verification(&self, data: FetchVerification) -> Result<String> {
        data
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

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
    
    pub async fn verify_session(&self, session: Session) -> Result<Session> {
        self.collection.find_one(
            doc! {
                "_id": &session.user_id,
                "password": &session.session_token
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or(Error::InvalidSession)?;

        Ok(session)
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
            header_user_id, header_session_token
        ) {
            (Some(auth), Some(user_id), Some(session_token)) => {
                let session = Session {
                    user_id,
                    session_token
                };

                if let Ok(session) = auth
                    .verify_session(session)
                    .await {
                    Outcome::Success(session)
                } else {
                    Outcome::Failure((Status::Forbidden, Error::InvalidSession))
                }
            }
            (None, _, _) => {
                Outcome::Failure((Status::InternalServerError, Error::InternalError))
            }
            (_, _, _) => {
                Outcome::Failure((Status::Forbidden, Error::MissingHeaders))
            }
        }
    }
}
