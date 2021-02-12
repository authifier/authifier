use crate::auth::{Auth, Session};
use crate::util::Error;
use crate::ARGON_CONFIG;

use mongodb::bson::doc;
use nanoid::nanoid;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ChangePassword {
    #[validate(length(min = 8, max = 72))]
    password: String,
    #[validate(length(min = 8, max = 72))]
    new_password: String,
}

#[post("/change/password", data = "<data>")]
pub async fn change_password(
    auth: State<'_, Auth>,
    session: Session,
    data: Json<ChangePassword>,
) -> crate::util::Result<()> {
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    auth.verify_password(&session, data.password.clone())
        .await?;

    let hash = argon2::hash_encoded(
        data.new_password.as_bytes(),
        nanoid!(24).as_bytes(),
        &ARGON_CONFIG,
    )
    .map_err(|_| Error::InternalError)?;

    auth.collection
        .update_one(
            doc! {
                "_id": &session.user_id,
            },
            doc! {
                "$set": {
                    "password": &hash
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "accounts",
        })?;

    Ok(())
}
