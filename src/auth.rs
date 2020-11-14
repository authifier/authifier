use super::db::AccountShort;
use super::util::{Error, Result};

use argon2::{self, Config};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::Collection;
use nanoid::nanoid;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

pub struct Auth {
    collection: Collection,
}

lazy_static! {
    static ref ARGON_CONFIG: Config<'static> = Config::default();
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Session {
    #[validate(length(min = 26, max = 26))]
    pub id: Option<String>,
    #[validate(length(min = 26, max = 26))]
    pub user_id: String,
    #[validate(length(min = 64, max = 64))]
    pub session_token: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Create {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 72))]
    password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Verify {
    #[validate(length(min = 32, max = 32))]
    pub code: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct FetchVerification {
    #[validate(email)]
    email: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Login {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 72))]
    password: String,
    #[validate(length(min = 0, max = 72))]
    device_name: Option<String>,
}

impl Auth {
    pub fn new(collection: Collection) -> Auth {
        Auth { collection }
    }

    pub async fn create_account(&self, data: Create) -> Result<String> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        if self
            .collection
            .find_one(
                doc! {
                    "email": &data.email
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError)?
            .is_some()
        {
            Err(Error::EmailInUse)?
        }

        let hash = argon2::hash_encoded(
            data.password.as_bytes(),
            Ulid::new().to_string().as_bytes(),
            &ARGON_CONFIG,
        )
        .map_err(|_| Error::InternalError)?;

        let user_id = Ulid::new().to_string();
        self.collection
            .insert_one(
                doc! {
                    "_id": &user_id,
                    "email": data.email,
                    "password": hash,
                    "verification": {
                        "verified": true
                    },
                    "sessions": []
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(user_id)
    }

    pub async fn verify_account(&self, data: Verify) -> Result<()> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        unimplemented!()
    }

    pub async fn fetch_verification(&self, data: FetchVerification) -> Result<String> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        unimplemented!()
    }

    pub async fn login(&self, data: Login) -> Result<Session> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        let user = self
            .collection
            .find_one(
                doc! {
                    "email": data.email
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "password": 1,
                        "verification.verified": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError)?
            .ok_or(Error::UnknownUser)?;

        if !user
            .get_document("verification")
            .map_err(|_| Error::DatabaseError)?
            .get_bool("verified")
            .map_err(|_| Error::DatabaseError)?
        {
            Err(Error::UnverifiedAccount)?
        }

        let user_id = user
            .get_str("_id")
            .map_err(|_| Error::DatabaseError)?
            .to_string();

        if !argon2::verify_encoded(
            user.get_str("password").map_err(|_| Error::DatabaseError)?,
            data.password.as_bytes(),
        )
        .map_err(|_| Error::InternalError)?
        {
            Err(Error::WrongPassword)?
        }

        let id = Ulid::new().to_string();
        let session_token = nanoid!(64);
        self.collection.update_one(
            doc! {
                "_id": &user_id
            },
            doc! {
                "$push": {
                    "sessions": {
                        "id": &id,
                        "token": &session_token,
                        "friendly_name": data.device_name.unwrap_or_else(|| "Unknown device.".to_string())
                    }
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError)?;

        Ok(Session {
            id: Some(id),
            user_id,
            session_token,
        })
    }

    pub async fn get_account(&self, session: Session) -> Result<AccountShort> {
        let user = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "email": 1,
                        "verification.verified": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError)?
            .ok_or(Error::UnknownUser)?;

        Ok(AccountShort {
            id: user
                .get_str("_id")
                .map_err(|_| Error::DatabaseError)?
                .to_string(),
            email: user
                .get_str("email")
                .map_err(|_| Error::DatabaseError)?
                .to_string(),
            verified: user
                .get_document("verification")
                .map_err(|_| Error::DatabaseError)?
                .get_bool("verified")
                .map_err(|_| Error::DatabaseError)?,
        })
    }

    pub async fn verify_session(&self, mut session: Session) -> Result<Session> {
        let doc = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "sessions.$": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError)?
            .ok_or(Error::InvalidSession)?;

        session.id = Some(
            doc.get_array("sessions")
                .map_err(|_| Error::DatabaseError)?
                .into_iter()
                .next()
                .ok_or(Error::DatabaseError)?
                .as_document()
                .ok_or(Error::DatabaseError)?
                .get_str("id")
                .map_err(|_| Error::DatabaseError)?
                .to_string(),
        );

        Ok(session)
    }

    pub async fn fetch_all_sessions(
        &self,
        session: Session,
    ) -> Result<Vec<super::db::AccountSessionInfo>> {
        let user = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
                },
                FindOneOptions::builder()
                    .projection(doc! { "sessions": 1 })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError)?
            .ok_or(Error::InvalidSession)?;

        user.get_array("sessions")
            .map_err(|_| Error::DatabaseError)?
            .into_iter()
            .map(|x| mongodb::bson::from_bson(x.clone()).map_err(|_| Error::InternalError))
            .collect()
    }

    pub async fn deauth_session(&self, session: Session, target: String) -> Result<()> {
        if self
            .collection
            .update_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
                },
                doc! {
                    "$pull": {
                        "sessions": {
                            "id": target
                        }
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError)?
            .modified_count
            == 0
        {
            Err(Error::OperationFailed)?
        }

        Ok(())
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
            header_user_id,
            header_session_token,
        ) {
            (Some(auth), Some(user_id), Some(session_token)) => {
                let session = Session {
                    id: None,
                    user_id,
                    session_token,
                };

                if let Ok(session) = auth.verify_session(session).await {
                    Outcome::Success(session)
                } else {
                    Outcome::Failure((Status::Forbidden, Error::InvalidSession))
                }
            }
            (None, _, _) => Outcome::Failure((Status::InternalServerError, Error::InternalError)),
            (_, _, _) => Outcome::Failure((Status::Forbidden, Error::MissingHeaders)),
        }
    }
}
