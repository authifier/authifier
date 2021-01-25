use crate::db::AccountShort;
use crate::auth::{Auth, Session};
use crate::util::{Error, Result};

use rocket::State;
use rocket_contrib::json::JsonValue;
use mongodb::options::FindOneOptions;
use mongodb::bson::{doc, from_document};

impl Auth {
    pub async fn get_account(&self, session: Session) -> Result<AccountShort> {
        let user = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
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
            .map_err(|_| Error::DatabaseError)?
            .ok_or(Error::UnknownUser)?;

        Ok(from_document(user).map_err(|_| Error::DatabaseError)?)
    }
}

#[get("/user")]
pub async fn fetch_account(
    auth: State<'_, Auth>,
    session: Session,
) -> crate::util::Result<JsonValue> {
    Ok(json!(auth.get_account(session).await?))
}
