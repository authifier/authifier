//! Verify an account
//! POST /verify/<code>
use rauth::{models::EmailVerification, util::normalise_email, RAuth, Result};
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

    // Update account email
    if let EmailVerification::Moving { new_email, .. } = &account.verification {
        account.email = new_email.clone();
        account.email_normalised = normalise_email(new_email.clone());
    }

    // Mark as verified
    account.verification = EmailVerification::Verified;

    // Save to database
    account.save(rauth).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use chrono::Duration;
    use iso8601_timestamp::Timestamp;

    use crate::test::*;

    #[async_std::test]
    async fn success() {
        let (rauth, _, mut account) = for_test_authenticated("verify_email::success").await;

        account.verification = EmailVerification::Pending {
            token: "token".into(),
            expiry: Timestamp::from_unix_timestamp_ms(
                chrono::Utc::now()
                    .checked_add_signed(Duration::seconds(100))
                    .expect("failed to checked_add_signed")
                    .timestamp_millis(),
            ),
        };

        account.save(&rauth).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::account::verify_email::verify_email],
        )
        .await;

        let res = client.post("/verify/token").dispatch().await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[async_std::test]
    async fn fail_invalid_token() {
        let rauth = for_test("verify_email::fail_invalid_token").await;

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::account::verify_email::verify_email],
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
