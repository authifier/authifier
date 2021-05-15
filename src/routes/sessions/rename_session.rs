use crate::auth::{Auth, Session};
use crate::util::{Error, Result};

use mongodb::bson::doc;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Data {
    #[validate(length(min = 0, max = 72))]
    device_name: String
}

impl Auth {
    pub async fn rename_session(&self, session: Session, data: Data) -> Result<()> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        self.collection.update_one(
            doc! {
                "_id": session.user_id,
                "session.id": session.id.unwrap()
            },
            doc! {
                "$set": {
                    "sessions.$.friendly_name": data.device_name
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError { operation: "update_one", with: "account" })?;

        Ok(())
    }
}

#[post("/rename_session", data = "<data>")]
pub async fn rename_session(
    auth: State<'_, Auth>,
    session: Session,
    data: Json<Data>,
) -> crate::util::Result<()> {
    auth.inner().rename_session(session, data.into_inner()).await?;
    Ok(())
}
