//! Send a password reset email
//! POST /account/reset_password
use authifier::util::normalise_email;
use authifier::{Authifier, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Reset Information
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataSendPasswordReset {
    /// Email associated with the account
    pub email: String,
    /// Captcha verification code
    pub captcha: Option<String>,
}

/// # Send Password Reset
///
/// Send an email to reset account password.
#[openapi(tag = "Account")]
#[post("/reset_password", data = "<data>")]
pub async fn send_password_reset(
    authifier: &State<Authifier>,
    data: Json<DataSendPasswordReset>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Check Captcha token
    authifier.config.captcha.check(data.captcha).await?;

    // Make sure email is valid and not blocked
    authifier
        .config
        .email_block_list
        .validate_email(&data.email)?;

    // From this point on, do not report failure to the
    // remote client, as this will open us up to user enumeration.

    // Normalise the email
    let email_normalised = normalise_email(data.email);

    // Try to find the relevant account
    if let Some(mut account) = authifier
        .database
        .find_account_by_normalised_email(&email_normalised)
        .await?
    {
        account.start_password_reset(authifier).await?;
    }

    // Never fail this route, (except for db error)
    // You may open the application to email enumeration otherwise.
    Ok(EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[tokio::test]
    async fn success() {
        let (authifier, _) =
            for_test_with_config("send_password_reset::success", test_smtp_config().await).await;

        Account::new(
            &authifier,
            "password_reset@smtp.test".into(),
            "password".into(),
            false,
        )
        .await
        .unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![
                crate::routes::account::password_reset::password_reset,
                crate::routes::account::send_password_reset::send_password_reset,
                crate::routes::session::login::login
            ],
        )
        .await;

        let res = client
            .post("/reset_password")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "password_reset@smtp.test",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let mail = assert_email_sendria("password_reset@smtp.test".into()).await;
        let res = client
            .patch("/reset_password")
            .header(ContentType::JSON)
            .body(
                json!({
                    "token": mail.code.expect("`code`"),
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
                    "email": "password_reset@smtp.test",
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }
}
