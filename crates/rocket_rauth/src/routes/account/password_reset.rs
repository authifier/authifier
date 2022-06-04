//! Confirm a password reset.
//! PATCH /account/reset_password
use rauth::util::hash_password;
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Password Reset
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataPasswordReset {
    /// Reset token
    pub token: String,
    /// New password
    pub password: String,
}

/// # Password Reset
///
/// Confirm password reset and change the password.
#[openapi(tag = "Account")]
#[patch("/reset_password", data = "<data>")]
pub async fn password_reset(
    rauth: &State<RAuth>,
    data: Json<DataPasswordReset>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Find the relevant account
    let mut account = rauth
        .database
        .find_account_with_password_reset(&data.token)
        .await?;

    // Verify password can be used
    rauth
        .config
        .password_scanning
        .assert_safe(&data.password)
        .await?;

    // Update the account
    account.password = hash_password(data.password)?;
    account.password_reset = None;

    // Commit to database
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
        let (rauth, _, mut account) = for_test_authenticated("password_reset::success").await;

        account.password_reset = Some(PasswordReset {
            token: "token".into(),
            expiry: Timestamp::from_unix_timestamp_ms(
                chrono::Utc::now()
                    .checked_add_signed(Duration::seconds(100))
                    .expect("failed to checked_add_signed")
                    .timestamp_millis(),
            ),
        });

        account.save(&rauth).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![
                crate::routes::account::password_reset::password_reset,
                crate::routes::session::login::login
            ],
        )
        .await;

        let res = client
            .patch("/reset_password")
            .header(ContentType::JSON)
            .body(
                json!({
                    "token": "token",
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "email@revolt.chat",
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }

    #[async_std::test]
    async fn fail_invalid_token() {
        let rauth = for_test("password_reset::fail_invalid_token").await;

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::account::password_reset::password_reset],
        )
        .await;

        let res = client
            .patch("/reset_password")
            .header(ContentType::JSON)
            .body(
                json!({
                    "token": "invalid",
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidToken\"}".into())
        );
    }
}
