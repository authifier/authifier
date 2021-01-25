use crate::auth::Auth;
use crate::util::{Error, Result};
use crate::options::EmailVerification;

use rocket::State;
use serde::Deserialize;
use validator::Validate;
use rocket::response::Redirect;

#[derive(Debug, Validate, Deserialize)]
pub struct Verify {
    #[validate(length(min = 32, max = 32))]
    pub code: String,
}

impl Auth {
    pub async fn verify_account(&self, data: Verify) -> Result<()> {
        data.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        unimplemented!()
    }
}

#[get("/verify/<code>")]
pub async fn verify_account(auth: State<'_, Auth>, code: String) -> crate::util::Result<Redirect> {
    auth.inner().verify_account(Verify { code }).await?;

    if let EmailVerification::Enabled { success_redirect_uri, .. } = &auth.options.email_verification {
        Ok(Redirect::to(success_redirect_uri.clone()))
    } else {
        unreachable!()
    }
}
