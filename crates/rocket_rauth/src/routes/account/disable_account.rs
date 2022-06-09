//! Disable an account.
//! POST /account/disable
use rauth::{
    models::{Account, ValidatedTicket},
    RAuth, Result,
};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Disable Account
///
/// Disable an account.
#[openapi(tag = "Account")]
#[post("/disable")]
pub async fn disable_account(
    rauth: &State<RAuth>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<EmptyResponse> {
    account.disable(rauth).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, account) = for_test_authenticated("disable_account::success").await;
        let ticket = MFATicket::new(&rauth, account.id.to_string(), true)
            .await
            .unwrap();
        let client = bootstrap_rocket_with_auth(
            rauth.clone(),
            routes![crate::routes::account::disable_account::disable_account],
        )
        .await;

        let res = client
            .post("/disable")
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert!(
            rauth
                .database
                .find_account(&account.id)
                .await
                .unwrap()
                .disabled
        );
        assert_eq!(
            rauth.database.find_session(&session.id).await.unwrap_err(),
            Error::UnknownUser
        );
    }
}
