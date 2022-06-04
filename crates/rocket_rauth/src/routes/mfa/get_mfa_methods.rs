//! Fetch available MFA methods.
//! GET /mfa/methods
use rauth::models::{Account, MFAMethod};
use rocket::serde::json::Json;

#[openapi(tag = "MFA")]
#[get("/methods")]
pub async fn get_mfa_methods(account: Account) -> Json<Vec<MFAMethod>> {
    Json(account.mfa.get_methods())
}

// TODO: write tests
