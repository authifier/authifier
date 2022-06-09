//! Fetch recovery codes for an account.
//! POST /mfa/recovery
use rauth::{
    models::{Account, ValidatedTicket},
    Result,
};
use rocket::serde::json::Json;

/// # Fetch Recovery Codes
///
/// Fetch recovery codes for an account.
#[openapi(tag = "MFA")]
#[post("/recovery")]
pub async fn fetch_recovery(
    account: Account,
    _ticket: ValidatedTicket,
) -> Result<Json<Vec<String>>> {
    Ok(Json(account.mfa.recovery_codes))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, account) = for_test_authenticated("fetch_recovery::success").await;
        let ticket = MFATicket::new(&rauth, account.id, true).await.unwrap();
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::mfa::fetch_recovery::fetch_recovery],
        )
        .await;

        let res = client
            .post("/recovery")
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(
            serde_json::from_str::<Vec<String>>(&res.into_string().await.unwrap())
                .unwrap()
                .is_empty()
        );
    }
}
