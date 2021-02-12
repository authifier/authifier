use crate::auth::{Auth, Session};
use crate::util::{Error, Result};

use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use nanoid::nanoid;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

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
                        "verification": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidCredentials)?;

        if user
            .get_document("verification")
            .map_err(|_| Error::DatabaseError {
                operation: "get_document(verification)",
                with: "account",
            })?
            .get_str("status")
            .map_err(|_| Error::DatabaseError {
                operation: "get_str(status)",
                with: "account",
            })?
            == "Pending"
        {
            return Err(Error::UnverifiedAccount);
        }

        let user_id = user
            .get_str("_id")
            .map_err(|_| Error::DatabaseError {
                operation: "get_str(_id)",
                with: "account",
            })?
            .to_string();

        if !argon2::verify_encoded(
            user.get_str("password").map_err(|_| Error::DatabaseError {
                operation: "get_str(password)",
                with: "account",
            })?,
            data.password.as_bytes(),
        )
        .map_err(|_| Error::InternalError)?
        {
            return Err(Error::InvalidCredentials);
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
        .map_err(|_| Error::DatabaseError { operation: "update_one", with: "account" })?;

        Ok(Session {
            id: Some(id),
            user_id,
            session_token,
        })
    }
}

#[post("/login", data = "<data>")]
pub async fn create_session(
    auth: State<'_, Auth>,
    data: Json<Login>,
) -> crate::util::Result<JsonValue> {
    Ok(json!(auth.inner().login(data.into_inner()).await?))
}
