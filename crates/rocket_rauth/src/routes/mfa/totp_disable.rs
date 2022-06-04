//! Disable TOTP 2FA.
//! DELETE /mfa/totp
use rauth::models::totp::Totp;
use rauth::models::{Account, ValidatedTicket};
use rauth::{RAuth, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "MFA")]
#[delete("/totp")]
pub async fn totp_disable(
    rauth: &State<RAuth>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<EmptyResponse> {
    // Disable TOTP
    account.mfa.totp_token = Totp::Disabled;

    // Save model to database
    account.save(rauth).await.map(|_| EmptyResponse)
}

// TODO: write tests
