//! Create a new account
//! POST /account/create
use rauth::models::Account;
use rauth::{Error, RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Account Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataCreateAccount {
    /// Valid email address
    pub email: String,
    /// Password
    pub password: String,
    /// Invite code
    pub invite: Option<String>,
    /// Captcha verification code
    pub captcha: Option<String>,
}

/// # Create Account
///
/// Create a new account.
#[openapi(tag = "Account")]
#[post("/create", data = "<data>")]
pub async fn create_account(
    rauth: &State<RAuth>,
    data: Json<DataCreateAccount>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Check Captcha token
    rauth.config.captcha.check(data.captcha).await?;

    // Make sure email is valid and not blocked
    rauth.config.email_block_list.validate_email(&data.email)?;

    // Ensure password is safe to use
    rauth
        .config
        .password_scanning
        .assert_safe(&data.password)
        .await?;

    // If required, fetch valid invite
    let invite = if rauth.config.invite_only {
        if let Some(invite) = data.invite {
            Some(rauth.database.find_invite(&invite).await?)
        } else {
            return Err(Error::MissingInvite);
        }
    } else {
        None
    };

    // Create account
    let account = Account::new(rauth, data.email, data.password, true).await?;

    // Use up the invite
    if let Some(mut invite) = invite {
        invite.claimed_by = Some(account.id);
        invite.used = true;

        rauth.database.save_invite(&invite).await?;
    }

    Ok(EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        let client = bootstrap_rocket(
            "create_account",
            "success",
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_email() {
        let client = bootstrap_rocket(
            "create_account",
            "fail_invalid_email",
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "invalid",
                    "password": "valid password"
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

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_password() {
        let client = bootstrap_rocket(
            "create_account",
            "fail_invalid_password",
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"CompromisedPassword\"}".into())
        );
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_invite() {
        let config = Config {
            invite_only: true,
            ..Default::default()
        };

        let (_, auth) = for_test_with_config("create_account::fail_invalid_invite", config).await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "valid password",
                    "invite": "invalid"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidInvite\"}".into())
        );
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success_valid_invite() {
        let config = Config {
            invite_only: true,
            ..Default::default()
        };

        let (db, auth) = for_test_with_config("create_account::success_valid_invite", config).await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let mut invite = Invite {
            id: Some("invite".into()),
            used: None,
            claimed_by: None,
        };

        invite.save(&db, None).await.unwrap();

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "valid password",
                    "invite": "invite"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let invite = Invite::find_one(&db, doc! { "_id": "invite" }, None)
            .await
            .unwrap()
            .expect("Invite");

        assert_eq!(invite.used, Some(true));
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_missing_captcha() {
        use crate::config::Captcha;

        let config = Config {
            captcha: Captcha::HCaptcha {
                secret: "0x0000000000000000000000000000000000000000".into(),
            },
            ..Default::default()
        };

        let (_, auth) = for_test_with_config("create_account::fail_missing_captcha", config).await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "valid password",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"CaptchaFailed\"}".into())
        );
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_captcha_invalid() {
        use crate::config::Captcha;

        let config = Config {
            captcha: Captcha::HCaptcha {
                secret: "0x0000000000000000000000000000000000000000".into(),
            },
            ..Default::default()
        };

        let (_, auth) = for_test_with_config("create_account::fail_missing_captcha", config).await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "valid password",
                    "captcha": "00000000-aaaa-bbbb-cccc-000000000000"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"CaptchaFailed\"}".into())
        );
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success_captcha_valid() {
        use crate::config::Captcha;

        let config = Config {
            captcha: Captcha::HCaptcha {
                secret: "0x0000000000000000000000000000000000000000".into(),
            },
            ..Default::default()
        };

        let (_, auth) = for_test_with_config("create_account::fail_missing_captcha", config).await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::create_account::create_account],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "valid password",
                    "captcha": "20000000-aaaa-bbbb-cccc-000000000002"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success_smtp_sent() {
        let (_, auth) = for_test_with_config(
            "create_account::success_smtp_sent",
            test_smtp_config().await,
        )
        .await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::web::account::create_account::create_account,
                crate::web::account::verify_email::verify_email
            ],
        )
        .await;

        let res = client
            .post("/create")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "create_account@smtp.test",
                    "password": "valid password",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let mail = assert_email_sendria("create_account@smtp.test".into()).await;
        let res = client
            .post(format!("/verify/{}", mail.code.expect("`code`")))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
