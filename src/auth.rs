use super::options::{Options};
use super::util::{Error, Result};

use argon2::{self, Config};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::Collection;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub struct Auth {
    pub collection: Collection,
    pub options: Options,
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

/* #[derive(Debug, Validate, Deserialize)]
pub struct FetchVerification {
    #[validate(email)]
    email: String,
} */

impl Auth {
    pub fn new(collection: Collection, options: Options) -> Auth {
        Auth {
            collection,
            options,
        }
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
                .iter()
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
