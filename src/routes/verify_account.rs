use crate::auth::{Auth, Verify};

use rocket::State;
use rocket::response::Redirect;

#[get("/verify/<code>")]
pub async fn verify_account(auth: State<'_, Auth>, code: String) -> crate::util::Result<Redirect> {
    auth.inner().verify_account(Verify { code }).await?;
    Ok(Redirect::to(auth.options.verified_uri.clone()))
}
