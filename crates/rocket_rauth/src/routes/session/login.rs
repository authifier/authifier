//! Login to an account
//! POST /session/login
use rauth::models::Session;
use rauth::util::normalise_email;
use rauth::{Error, RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Login Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataLogin {
    /// Email
    pub email: String,

    /// Password
    pub password: Option<String>,
    /// UN-USED: MFA challenge
    pub challenge: Option<String>,

    /// Friendly name used for the session
    pub friendly_name: Option<String>,
    /// Captcha verification code
    pub captcha: Option<String>,
}

// TODO: remove dead_code
#[allow(dead_code)]
#[derive(Serialize, JsonSchema)]
#[serde(tag = "result")]
pub enum ResponseLogin {
    Success(Session),
    EmailOTP,
    MFA {
        ticket: String,
        // TODO: swap this out for an enum
        allowed_methods: Vec<String>,
    },
}

/// # Login
///
/// Login to an account.
#[openapi(tag = "Session")]
#[post("/login", data = "<data>")]
pub async fn login(rauth: &State<RAuth>, data: Json<DataLogin>) -> Result<Json<ResponseLogin>> {
    let data = data.into_inner();

    // Check Captcha token
    rauth.config.captcha.check(data.captcha).await?;

    // Generate a session name ahead of time.
    let name = data.friendly_name.unwrap_or_else(|| "Unknown".to_string());

    // Try to find the account we want.
    let email_normalised = normalise_email(data.email);

    if let Some(account) = rauth
        .database
        .find_account_by_normalised_email(&email_normalised)
        .await?
    {
        // Figure out whether we are doing password, 1FA key or email 1FA OTP.
        if let Some(password) = data.password {
            // Make sure password has not been compromised
            rauth
                .config
                .password_scanning
                .assert_safe(&password)
                .await?;

            // Verify the password is correct.
            account.verify_password(&password)?;

            // Prevent disabled accounts from logging in.
            if account.disabled {
                return Err(Error::DisabledAccount);
            }

            Ok(Json(ResponseLogin::Success(
                account.create_session(rauth, name).await?,
            )))
        } else if let Some(_challenge) = data.challenge {
            // TODO: implement; issue #5
            Err(Error::InvalidCredentials)
        } else {
            // TODO: implement; issue #5
            Err(Error::InvalidCredentials)
        }
    } else {
        Err(Error::InvalidCredentials)
    }
}

#[cfg(test)]
#[cfg(feature = "test")]
#[cfg(feature = "TODO")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        let (_, auth) = for_test("login::success").await;

        auth.create_account("example@validemail.com".into(), "password".into(), false)
            .await
            .unwrap();

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::routes::session::login::login]).await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "EXAMPLE@validemail.com",
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }

    #[async_std::test]
    async fn fail_invalid_user() {
        let client = bootstrap_rocket(
            "create_account",
            "fail_invalid_user",
            routes![crate::routes::session::login::login],
        )
        .await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidCredentials\"}".into())
        );
    }

    #[async_std::test]
    async fn fail_disabled_account() {
        let (db, auth) = for_test("login::fail_disabled_account").await;

        let mut account = auth
            .create_account("example@validemail.com".into(), "password".into(), false)
            .await
            .unwrap();

        account.disabled = Some(true);
        account.save(&db, None).await.unwrap();

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::routes::session::login::login]).await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"DisabledAccount\"}".into())
        );
    }
}
