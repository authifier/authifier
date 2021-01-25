use crate::auth::{Auth, Create};

use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

#[post("/create", data = "<data>")]
pub async fn create_account(
    auth: State<'_, Auth>,
    data: Json<Create>,
) -> crate::util::Result<JsonValue> {
    Ok(json!({
        "user_id": auth.inner().create_account(data.into_inner()).await?
    }))
}
