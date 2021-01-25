use crate::auth::{Auth, Session};

use rocket::State;

#[get("/logout")]
pub async fn logout(auth: State<'_, Auth>, session: Session) -> crate::util::Result<()> {
    let id = session.id.clone().unwrap();
    auth.deauth_session(session, id).await
}
