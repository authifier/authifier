use crate::ARGON_CONFIG;
use crate::auth::Auth;
use crate::options::EmailVerification;
use crate::util::normalise_email;
use crate::util::{Error, Result};

use chrono::Utc;
use mongodb::bson::{doc, Bson};
use nanoid::nanoid;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Create {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 72))]
    password: String,
}

impl Auth {
    pub async fn create_account(&self, data: Create) -> Result<String> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        let normalised = self.check_email_is_use(data.email.clone()).await?;
        let hash = argon2::hash_encoded(
            data.password.as_bytes(),
            nanoid!(24).as_bytes(),
            &ARGON_CONFIG,
        )
        .map_err(|_| Error::InternalError)?;

        let verification = if let EmailVerification::Enabled {
            smtp,
            verification_expiry,
            ..
        } = &self.options.email_verification
        {
            let token = nanoid!(32);
            self.email_send_verification(&smtp, &data.email, &token)?;

            doc! {
                "status": "Pending",
                "token": token,
                "expiry": Bson::DateTime(Utc::now() + *verification_expiry)
            }
        } else {
            doc! {
                "status": "Verified"
            }
        };

        let user_id = Ulid::new().to_string();
        self.collection
            .insert_one(
                doc! {
                    "_id": &user_id,
                    "email": &data.email,
                    "email_normalised": normalised,
                    "password": hash,
                    "verification": verification,
                    "sessions": []
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "account",
            })?;

        Ok(user_id)
    }
}

#[post("/create", data = "<data>")]
pub async fn create_account(
    auth: State<'_, Auth>,
    data: Json<Create>,
) -> crate::util::Result<JsonValue> {
    Ok(json!({
        "user_id": auth.inner().create_account(data.into_inner()).await?
    }))
}
