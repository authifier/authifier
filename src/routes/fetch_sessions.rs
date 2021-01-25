use crate::auth::{Auth, Session};

use rocket::State;
use rocket_contrib::json::JsonValue;

#[get("/sessions")]
pub async fn fetch_sessions(auth: State<'_, Auth>, session: Session) -> crate::util::Result<JsonValue> {
    Ok(json!(auth.fetch_all_sessions(session).await?))
}
