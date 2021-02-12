use crate::auth::{Auth, Session};
use crate::util::{Error, Result};

use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use rocket::State;
use rocket_contrib::json::JsonValue;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSessionInfo {
    pub id: String,
    pub friendly_name: String,
}

impl Auth {
    pub async fn fetch_all_sessions(&self, session: Session) -> Result<Vec<AccountSessionInfo>> {
        let user = self
            .collection
            .find_one(
                doc! {
                    "_id": &session.user_id
                },
                FindOneOptions::builder()
                    .projection(doc! { "sessions": 1 })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidSession)?;

        user.get_array("sessions")
            .map_err(|_| Error::DatabaseError {
                operation: "get_array(sessions)",
                with: "account",
            })?
            .iter()
            .map(|x| {
                mongodb::bson::from_bson(x.clone()).map_err(|_| Error::DatabaseError {
                    operation: "from_bson",
                    with: "array(sessions)",
                })
            })
            .collect()
    }
}

#[get("/sessions")]
pub async fn fetch_sessions(
    auth: State<'_, Auth>,
    session: Session,
) -> crate::util::Result<JsonValue> {
    Ok(json!(auth.fetch_all_sessions(session).await?))
}
