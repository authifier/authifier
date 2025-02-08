//! Disable TOTP 2FA.
//! DELETE /mfa/totp
use authifier::models::totp::Totp;
use authifier::models::{Account, AuthFlow, ValidatedTicket};
use authifier::{Authifier, Error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Disable TOTP 2FA
///
/// Disable TOTP 2FA for an account.
#[openapi(tag = "MFA")]
#[delete("/totp")]
pub async fn totp_disable(
    authifier: &State<Authifier>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<EmptyResponse> {
    let AuthFlow::Password(auth) = &mut account.auth_flow else {
        return Err(Error::NotAvailable);
    };

    // Disable TOTP
    auth.mfa.totp_token = Totp::Disabled;

    // Save model to database
    account.save(authifier).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (authifier, session, account, _) =
            for_test_authenticated("totp_disable::success").await;
        let ticket = MFATicket::new(account.id, true);
        ticket.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![crate::routes::mfa::totp_disable::totp_disable],
        )
        .await;

        let res = client
            .delete("/totp")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
