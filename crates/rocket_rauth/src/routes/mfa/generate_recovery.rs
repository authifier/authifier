//! Re-generate recovery codes for an account.
//! PATCH /mfa/recovery
use rauth::models::{Account, ValidatedTicket};
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Generate Recovery Codes
///
/// Re-generate recovery codes for an account.
#[openapi(tag = "MFA")]
#[patch("/recovery")]
pub async fn generate_recovery(
    rauth: &State<RAuth>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<Json<Vec<String>>> {
    // Generate new codes
    account.mfa.generate_recovery_codes();

    // Save account model
    account.save(rauth).await?;

    // Return them to the user
    Ok(Json(account.mfa.recovery_codes))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, account) = for_test_authenticated("generate_recovery::success").await;
        let ticket1 = MFATicket::new(&rauth, account.id.to_string(), true)
            .await
            .unwrap();
        let ticket2 = MFATicket::new(&rauth, account.id, true).await.unwrap();
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![
                crate::routes::mfa::generate_recovery::generate_recovery,
                crate::routes::mfa::fetch_recovery::fetch_recovery
            ],
        )
        .await;

        let res = client
            .patch("/recovery")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .header(Header::new("X-MFA-Ticket", ticket1.token))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Vec<String>>(&res.into_string().await.unwrap()).is_ok());

        let res = client
            .post("/recovery")
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket2.token))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            serde_json::from_str::<Vec<String>>(&res.into_string().await.unwrap())
                .unwrap()
                .len(),
            10
        );
    }
}
