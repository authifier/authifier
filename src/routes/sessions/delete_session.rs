use crate::auth::{Auth, Session};

use rocket::State;

#[delete("/sessions/<id>")]
pub async fn delete_session(
    auth: &State<Auth>,
    session: Session,
    id: String,
) -> crate::util::Result<()> {
    auth.deauth_session(session, id).await
}
