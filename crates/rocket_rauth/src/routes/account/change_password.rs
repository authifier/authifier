//! Change account password.
//! PATCH /account/change/password
use rauth::models::Account;
use rauth::util::hash_password;
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Change Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataChangePassword {
    /// New password
    pub password: String,
    /// Current password
    pub current_password: String,
}

/// # Change Password
///
/// Change the current account password.
#[openapi(tag = "Account")]
#[patch("/change/password", data = "<data>")]
pub async fn change_password(
    rauth: &State<RAuth>,
    mut account: Account,
    data: Json<DataChangePassword>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Verify password can be used
    rauth
        .config
        .password_scanning
        .assert_safe(&data.password)
        .await?;

    // Ensure given password is correct
    account.verify_password(&data.current_password)?;

    // Hash and replace password
    account.password = hash_password(data.password)?;

    // Commit to database
    account.save(rauth).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, _) = for_test_authenticated("change_password::success").await;
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::account::change_password::change_password],
        )
        .await;

        let res = client
            .patch("/change/password")
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "password": "new password",
                    "current_password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let res = client
            .patch("/change/password")
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token))
            .body(
                json!({
                    "password": "sussy password",
                    "current_password": "new password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
