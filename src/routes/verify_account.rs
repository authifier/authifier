use crate::{auth::{Auth, Verify}, options::EmailVerification};

use rocket::response::Redirect;
use rocket::State;

#[get("/verify/<code>")]
pub async fn verify_account(auth: State<'_, Auth>, code: String) -> crate::util::Result<Redirect> {
    auth.inner().verify_account(Verify { code }).await?;

    if let EmailVerification::Enabled { success_redirect_uri, .. } = &auth.options.email_verification {
        Ok(Redirect::to(success_redirect_uri.clone()))
    } else {
        unreachable!()
    }
}
