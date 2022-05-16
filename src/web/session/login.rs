/// Login to an account
/// POST /session/login
use mongodb::options::{Collation, FindOneOptions};
use rocket::serde::json::Json;
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{Error, Result};

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
pub async fn login(auth: &State<Auth>, data: Json<DataLogin>) -> Result<Json<ResponseLogin>> {
    let data = data.into_inner();

    // Perform validation on given data.
    auth.check_captcha(data.captcha).await?;
    auth.validate_email(&data.email).await?;

    // Generate a session name ahead of time.
    let name = data.friendly_name.unwrap_or_else(|| "Unknown".to_string());

    // * We could check if passwords are compromised
    // * on login, in the future.
    // auth.validate_password(&password).await?;

    // Try to find the account we want.
    if let Some(account) = Account::find_one(
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
        // Figure out whether we are doing password, 1FA key or email 1FA OTP.
        if let Some(password) = data.password {
            // Verify the password is correct.
            account.verify_password(&password)?;

            // Prevent disabled accounts from logging in.
            if account.disabled.unwrap_or(false) {
                return Err(Error::DisabledAccount);
            }

            Ok(Json(ResponseLogin::Success(
                auth.create_session(&account, name).await?,
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
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        let (_, auth) = for_test("login::success").await;

        auth.create_account("example@validemail.com".into(), "password".into(), false)
            .await
            .unwrap();

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::login::login]).await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "EXAMPLE@validemail.com",
                    "password": "password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_user() {
        let client = bootstrap_rocket(
            "create_account",
            "fail_invalid_user",
            routes![crate::web::session::login::login],
        )
        .await;

        let res = client
            .post("/login")
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

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidCredentials\"}".into())
        );
    }

    #[cfg(feature = "async-std-runtime")]
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
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::login::login]).await;

        let res = client
            .post("/login")
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

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"DisabledAccount\"}".into())
        );
    }
}
