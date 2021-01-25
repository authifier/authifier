use crate::auth::{Auth, Session};

use rocket::State;
use rocket_contrib::json::JsonValue;

#[get("/user")]
pub async fn fetch_account(auth: State<'_, Auth>, session: Session) -> crate::util::Result<JsonValue> {
    Ok(json!(auth.get_account(session).await?))
}
