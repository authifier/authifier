use crate::auth::Auth;
use crate::db::AccountVerification;
use crate::options::EmailVerification;
use crate::util::Error;

use chrono::Utc;
use mongodb::{
    bson::{doc, from_document, Bson},
    options::FindOneOptions,
};
use nanoid::nanoid;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ResendVerification {
    #[validate(email)]
    email: String,
}

#[post("/resend", data = "<data>")]
pub async fn resend_verification(
    auth: State<'_, Auth>,
    data: Json<ResendVerification>,
) -> crate::util::Result<()> {
    let doc = auth
        .collection
        .find_one(
            doc! {
                "email": &data.email,
                "verification.status": "Pending"
            },
            FindOneOptions::builder()
                .projection(doc! {
                    "_id": 1,
                    "verification": 1
                })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "account",
        })?
        .ok_or_else(|| Error::UnknownUser)?;

    let document = doc
        .get_document("verification")
        .map_err(|_| Error::DatabaseError {
            operation: "get_document(verification)",
            with: "account",
        })?;

    let verification: AccountVerification =
        from_document(document.clone()).map_err(|_| Error::DatabaseError {
            operation: "from_document(verification)",
            with: "account",
        })?;

    if let AccountVerification::Pending { .. } = verification {
        if let EmailVerification::Enabled {
            smtp,
            templates,
            verification_expiry,
            ..
        } = &auth.options.email_verification
        {
            let token = nanoid!(32);
            auth.email_send_verification(&smtp, &templates, &data.email, &token)?;

            auth.collection.update_one(
                doc! {
                    "_id": doc.get_str("_id")
                        .map_err(|_| Error::DatabaseError { operation: "get_str(_id)", with: "account" })?
                },
                doc! {
                    "$set": {
                        "verification": {
                            "status": "Pending",
                            "token": token,
                            "expiry": Bson::DateTime(Utc::now() + *verification_expiry)
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
        Err(Error::UnknownUser)
    }
}
