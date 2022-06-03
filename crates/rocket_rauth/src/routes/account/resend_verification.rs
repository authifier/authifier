//! Resend account verification email
//! POST /account/reverify
use rauth::{util::normalise_email, RAuth, Result};
use rocket::{serde::json::Json, State};
use rocket_empty::EmptyResponse;

/// # Resend Information
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataResendVerification {
    /// Email associated with the account
    pub email: String,
    /// Captcha verification code
    pub captcha: Option<String>,
}

/// # Resend Verification
///
/// Resend account creation verification email.
#[openapi(tag = "Account")]
#[post("/reverify", data = "<data>")]
pub async fn resend_verification(
    rauth: &State<RAuth>,
    data: Json<DataResendVerification>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Check Captcha token
    rauth.config.captcha.check(data.captcha).await?;

    // Make sure email is valid and not blocked
    rauth.config.email_block_list.validate_email(&data.email)?;

    // From this point on, do not report failure to the
    // remote client, as this will open us up to user enumeration.

    // Normalise the email
    let email_normalised = normalise_email(data.email);

    // Try to find the relevant account
    if let Some(mut account) = rauth
        .database
        .find_account_by_normalised_email(&email_normalised)
        .await?
    {
        account.start_email_verification(rauth).await?;
    }

    // Never fail this route,
    // You may open the application to email enumeration otherwise.
    Ok(EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
#[cfg(feature = "TODO")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use chrono::Utc;
        use mongodb::bson::DateTime;

        let (db, auth) =
            for_test_with_config("resend_verification::success", test_smtp_config().await).await;

        let mut account = auth
            .create_account(
                "resend_verification@smtp.test".into(),
                "password".into(),
                false,
            )
            .await
            .unwrap();

        account.verification = AccountVerification::Pending {
            token: "".into(),
            expiry: DateTime(Utc::now()),
        };

        account.save(&db, None).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::web::account::resend_verification::resend_verification,
                crate::web::account::verify_email::verify_email
            ],
        )
        .await;

        let res = client
            .post("/reverify")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "resend_verification@smtp.test",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let mail = assert_email_sendria("resend_verification@smtp.test".into()).await;
        let res = client
            .post(format!("/verify/{}", mail.code.expect("`code`")))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[async_std::test]
    async fn success_unknown() {
        let (_, auth) = for_test_with_config(
            "resend_verification::success_unknown",
            test_smtp_config().await,
        )
        .await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::resend_verification::resend_verification],
        )
        .await;

        let res = client
            .post("/reverify")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "smtptest1@insrt.uk",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[async_std::test]
    async fn fail_bad_email() {
        let client = bootstrap_rocket(
            "resend_verification",
            "fail_bad_email",
            routes![crate::web::account::resend_verification::resend_verification],
        )
        .await;

        let res = client
            .post("/reverify")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "invalid",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"IncorrectData\",\"with\":\"email\"}".into())
        );
    }
}
