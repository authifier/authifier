use crate::auth::Auth;
use crate::util::Error;
use crate::{options::EmailVerification, ARGON_CONFIG};

use chrono::Utc;
use mongodb::bson::{doc, Bson};
use nanoid::nanoid;
use rocket::{response::Redirect, State};
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ResetPassword {
    #[validate(length(min = 32, max = 32))]
    token: String,
    #[validate(length(min = 8, max = 1024))]
    password: String,
}

#[post("/reset", data = "<data>")]
pub async fn reset_password(
    auth: State<'_, Auth>,
    data: Json<ResetPassword>,
) -> crate::util::Result<Redirect> {
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if auth
        .collection
        .find_one(
            doc! {
                "password_reset.token": &data.token,
                "password_reset.expiry": {
                    "$gte": Bson::DateTime(Utc::now())
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "account",
        })?
        .is_none()
    {
        return Err(Error::InvalidToken);
    }

    let hash = argon2::hash_encoded(
        data.password.as_bytes(),
        nanoid!(24).as_bytes(),
        &ARGON_CONFIG,
    )
    .map_err(|_| Error::InternalError)?;

    auth.collection
        .update_one(
            doc! {
                "password_reset.token": &data.token,
            },
            doc! {
                "$set": {
                    "password": &hash
                },
                "$unset": {
                    "password_reset": 1
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "account",
        })?;

    if let EmailVerification::Enabled {
        success_redirect_uri,
        ..
    } = &auth.options.email_verification
    {
        Ok(Redirect::to(success_redirect_uri.clone()))
    } else {
        unreachable!()
    }
}
