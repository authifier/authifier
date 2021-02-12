use crate::options::EmailVerification;
use crate::util::{Error, Result};
use crate::{auth::Auth, db::AccountVerification, util::normalise_email};

use mongodb::bson::doc;
use mongodb::{bson::from_document, options::FindOneOptions};
use rocket::response::Redirect;
use rocket::State;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Verify {
    #[validate(length(min = 32, max = 32))]
    pub code: String,
}

impl Auth {
    pub async fn verify_account(&self, data: Verify) -> Result<()> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        let doc = self
            .collection
            .find_one(
                doc! {
                    "verification.token": &data.code
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "email": 1,
                        "verification": 1
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or_else(|| Error::InvalidToken)?;

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

        let update = match verification {
            AccountVerification::Verified => unreachable!(),
            AccountVerification::Moving { new_email, .. } => {
                let normalised_email = normalise_email(new_email.clone());

                doc! {
                    "$set": {
                        "verification": {
                            "status": "Verified"
                        },
                        "email": new_email,
                        "email_normalised": normalised_email
                    }
                }
            }
            AccountVerification::Pending { .. } => {
                doc! {
                    "$set": {
                        "verification": {
                            "status": "Verified"
                        }
                    }
                }
            }
        };

        self.collection
            .update_one(
                doc! {
                    "verification.token": &data.code
                },
                update,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "account",
            })?;

        Ok(())
    }
}

#[get("/verify/<code>")]
pub async fn verify_account(auth: State<'_, Auth>, code: String) -> crate::util::Result<Redirect> {
    auth.inner().verify_account(Verify { code }).await?;

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
