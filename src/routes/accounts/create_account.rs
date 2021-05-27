use crate::auth::Auth;
use crate::options::EmailVerification;
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
    #[validate(length(min = 8, max = 1024))]
    password: String,
    invite: Option<String>,
    captcha: Option<String>,
}

impl Auth {
    pub async fn create_account(&self, data: Create) -> Result<String> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        self.verify_captcha(&data.captcha).await?;

        if let Some(col) = &self.options.invite_only_collection {
            if let Some(code) = &data.invite {
                if col
                    .find_one(
                        doc! {
                            "_id": code,
                            "used": {
                                "$ne": true
                            }
                        },
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "find_one",
                        with: "invites",
                    })?
                    .is_none()
                {
                    return Err(Error::InvalidInvite);
                }
            } else {
                return Err(Error::MissingInvite);
            }
        }

        let normalised = self.check_email_is_use(data.email.clone()).await?;
        let hash = self.hash_password(data.password.clone()).await?;

        let verification_token = nanoid!(32);
        let verification = if let EmailVerification::Enabled {
            verification_expiry,
            ..
        } = &self.options.email_verification
        {
            doc! {
                "status": "Pending",
                "token": &verification_token,
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

        if let Some(col) = &self.options.invite_only_collection {
            if let Some(code) = &data.invite {
                col.update_one(
                    doc! {
                        "_id": code
                    },
                    doc! {
                        "$set": {
                            "used": true,
                            "claimed_by": &user_id
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "invites",
                })?;
            }
        }
        
        if let EmailVerification::Enabled {
            smtp,
            templates,
            ..
        } = &self.options.email_verification
        {
            self.email_send_verification(&smtp, &templates, &data.email, &verification_token).ok();
            eprintln!("Failed to send an email to {}", &data.email);
        }

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
