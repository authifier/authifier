/// Create a new account
/// POST /account/create
use rocket::serde::json::Json;
use rocket::State;

use crate::logic::Auth;
use crate::util::{EmptyResponse, Result};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub email: String,
    pub password: String,
    pub invite: Option<String>,
    pub captcha: Option<String>,
}

#[post("/create", data = "<data>")]
pub async fn create_account(auth: &State<Auth>, data: Json<Data>) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Perform validation on given data.
    auth.check_captcha(data.captcha).await?;
    auth.validate_email(&data.email).await?;
    auth.validate_password(&data.password).await?;

    // Make sure the user has a valid invite if required.
    let invite = auth.check_invite(data.invite).await?;

    // Create an account but quietly fail any errors.
    let account = auth
        .create_account(data.email, data.password, true)
        .await
        .ok();

    // Make sure to use up the invite.
    if let Some(account) = account {
        if let Some(invite) = invite {
            invite.claim(&auth.db, account.id.unwrap()).await.ok();
        }
    }

    Ok(EmptyResponse)
}

#[cfg(test)]
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
