use crate::auth::Auth;
use crate::options::EmailVerification;
use crate::util::Error;

use chrono::Utc;
use mongodb::{
    bson::{doc, Bson},
    options::FindOneOptions,
};
use nanoid::nanoid;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ResetPassword {
    #[validate(email)]
    email: String,
}

#[post("/send_reset", data = "<data>")]
pub async fn send_password_reset(
    auth: State<'_, Auth>,
    data: Json<ResetPassword>,
) -> crate::util::Result<()> {
    if let Some(doc) = auth
        .collection
        .find_one(
            doc! {
                "email": &data.email,
                "verification.status": {
                    "$ne": "Pending"
                }
            },
            FindOneOptions::builder()
                .projection(doc! {
                    "_id": 1
                })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "account",
        })?
    {
        if let EmailVerification::Enabled {
            smtp,
            templates,
            password_reset_url,
            password_reset_expiry,
            ..
        } = &auth.options.email_verification
        {
            let token = nanoid!(32);
            auth.email_send_password_reset(
                &smtp,
                &templates,
                password_reset_url,
                &data.email,
                &token,
            )?;

            auth.collection.update_one(
                doc! {
                    "_id": doc.get_str("_id")
                        .map_err(|_| Error::DatabaseError { operation: "get_str(_id)", with: "account" })?
                },
                doc! {
                    "$set": {
                        "password_reset": {
                            "token": token,
                            "expiry": Bson::DateTime(Utc::now() + *password_reset_expiry)
                        }
                    }
                },
                None
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "account" })?;

            Ok(())
        } else {
            Err(Error::InternalError)
        }
    } else {
        Ok(())
    }
}
