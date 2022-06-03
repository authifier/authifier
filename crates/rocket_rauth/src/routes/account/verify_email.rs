//! Verify an account
//! POST /verify/<code>
use rauth::{models::EmailVerification, RAuth, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Verify Email
///
/// Verify an email address.
#[openapi(tag = "Account")]
#[post("/verify/<code>")]
pub async fn verify_email(rauth: &State<RAuth>, code: String) -> Result<EmptyResponse> {
    // Find the account
    let mut account = rauth
        .database
        .find_account_with_email_verification(&code)
        .await?;

    // Mark as verified
    account.verification = EmailVerification::Verified;

    // Save to database
    rauth
        .database
        .save_account(&account)
        .await
        .map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use chrono::{Duration, Utc};
        use mongodb::bson::DateTime;

        let (db, auth, _, mut account) = for_test_authenticated("verify_email::success").await;

        account.verification = AccountVerification::Pending {
            token: "token".into(),
            expiry: DateTime(
                Utc::now()
                    .checked_add_signed(Duration::seconds(60))
                    .expect("failed to checked_add_signed"),
            ),
        };

        account.save(&db, None).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::verify_email::verify_email],
        )
        .await;

        let res = client.post("/verify/token").dispatch().await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_token() {
        let (_, auth) = for_test("verify_email::fail_invalid_token").await;

        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::verify_email::verify_email],
        )
        .await;

        let res = client.post("/verify/token").dispatch().await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidToken\"}".into())
        );
    }
}
