use crate::auth::{Auth, Session};
use crate::util::Error;

use mongodb::bson::doc;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ChangePassword {
    #[validate(length(min = 8, max = 1024))]
    password: String,
    #[validate(length(min = 8, max = 1024))]
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

    let hash = auth.hash_password(data.new_password.clone()).await?;
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
