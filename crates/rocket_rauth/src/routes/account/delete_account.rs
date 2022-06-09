//! Delete an account.
//! POST /account/delete
use rauth::{
    models::{Account, ValidatedTicket},
    RAuth, Result,
};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete Account
///
/// Request to have an account deleted.
#[openapi(tag = "Account")]
#[post("/delete")]
pub async fn delete_account(
    rauth: &State<RAuth>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<EmptyResponse> {
    account
        .start_account_deletion(rauth)
        .await
        .map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, mut account) =
            for_test_authenticated_with_config("delete_account::success", test_smtp_config().await)
                .await;

        account.email = "delete_account@smtp.test".to_string();
        account.save(&rauth).await.unwrap();

        let ticket = MFATicket::new(&rauth, account.id, true).await.unwrap();
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![
                crate::routes::account::delete_account::delete_account,
                crate::routes::account::confirm_deletion::confirm_deletion
            ],
        )
        .await;

        let res = client
            .post("/delete")
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let mail = assert_email_sendria("delete_account@smtp.test".into()).await;
        let res = client
            .put("/delete")
            .header(ContentType::JSON)
            .body(
                json!({
                    "token": mail.code.expect("`code`")
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
