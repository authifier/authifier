use crate::auth::{Auth, Login};

use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

#[post("/login", data = "<data>")]
pub async fn create_session(auth: State<'_, Auth>, data: Json<Login>) -> crate::util::Result<JsonValue> {
    Ok(json!(auth.inner().login(data.into_inner()).await?))
}
