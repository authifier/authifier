use rauth::Result;
/// Resend account verification email
/// POST /account/reverify
use rocket::serde::json::Json;
use rocket::State;
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
    // auth: &State<Auth>,
    data: Json<DataResendVerification>,
) -> Result<EmptyResponse> {
    /*let data = data.into_inner();

    // Perform validation on given data.
    auth.check_captcha(data.captcha).await?;
    auth.validate_email(&data.email).await?;

    // From this point on, do not report failure to the
    // remote client, as this will open us up to user enumeration.

    // Try to find the relevant account.
    if let Some(mut account) = Account::find_one(
        &auth.db,
        doc! { "email": data.email },
        FindOneOptions::builder()
            .collation(Collation::builder().locale("en").strength(2).build())
            .build(),
    )
    .await
    .map_err(|_| Error::DatabaseError {
        operation: "find_one",
        with: "account",
    })? {
        if let AccountVerification::Pending { .. } = &account.verification {
            // Send out verification email and update verification object.
            account.verification = auth
                .generate_email_verification(account.email.clone())
                .await;

            // Commit to database.
            account.save_to_db(&auth.db).await?;
        }
    }

    // Never fail this route,
    // You may open the application to email enumeration otherwise.
    Ok(EmptyResponse)*/
    todo!()
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
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

    #[cfg(feature = "async-std-runtime")]
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

    #[cfg(feature = "async-std-runtime")]
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
