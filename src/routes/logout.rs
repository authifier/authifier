use crate::auth::{Auth, Session};
use crate::util::{Error, Result};

use rocket::State;
use mongodb::bson::doc;

impl Auth {
    pub async fn deauth_session(&self, session: Session, target: String) -> Result<()> {
        if self
            .collection
            .update_one(
                doc! {
                    "_id": &session.user_id,
                    "sessions.token": &session.session_token
                },
                doc! {
                    "$pull": {
                        "sessions": {
                            "id": target
                        }
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "account" })?
            .modified_count
            == 0
        {
            return Err(Error::OperationFailed);
        }

        Ok(())
    }
}

#[get("/logout")]
pub async fn logout(auth: State<'_, Auth>, session: Session) -> crate::util::Result<()> {
    let id = session.id.clone().unwrap();
    auth.deauth_session(session, id).await
}
