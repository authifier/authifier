//! Login to an account
//! POST /session/login
use rauth::models::{EmailVerification, MFAMethod, MFAResponse, MFATicket, Session};
use rauth::util::normalise_email;
use rauth::{Error, RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Login Data
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DataLogin {
    Email {
        /// Email
        email: String,
        /// Password
        password: String,
        /// Captcha verification code
        captcha: Option<String>,
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
    MFA {
        /// Unvalidated MFA ticket
        ///
        /// Used to resolve the correct account
        mfa_ticket: String,
        /// Valid MFA response
        ///
        /// This will take precedence over the `password` field where applicable
        mfa_response: MFAResponse,
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "result")]
pub enum ResponseLogin {
    Success(Session),
    MFA {
        ticket: String,
        allowed_methods: Vec<MFAMethod>,
    },
}

/// # Login
///
/// Login to an account.
#[openapi(tag = "Session")]
#[post("/login", data = "<data>")]
pub async fn login(rauth: &State<RAuth>, data: Json<DataLogin>) -> Result<Json<ResponseLogin>> {
    let (account, name) = match data.into_inner() {
        DataLogin::Email {
            email,
            password,
            captcha,
            friendly_name,
        } => {
            // Check Captcha token
            rauth.config.captcha.check(captcha).await?;

            // Try to find the account we want
            let email_normalised = normalise_email(email);

            // Lookup the email in database
            if let Some(account) = rauth
                .database
                .find_account_by_normalised_email(&email_normalised)
                .await?
            {
                // Make sure the account has been verified
                if let EmailVerification::Pending { .. } = account.verification {
                    return Err(Error::UnverifiedAccount);
                }

                // Make sure password has not been compromised
                rauth
                    .config
                    .password_scanning
                    .assert_safe(&password)
                    .await?;

                // Verify the password is correct.
                account.verify_password(&password)?;

                // Check whether an MFA step is required.
                if account.mfa.is_active() {
                    // Create a new ticket
                    let ticket = MFATicket::new(rauth, account.id, false).await?;

                    // Return applicable methods
                    return Ok(Json(ResponseLogin::MFA {
                        ticket: ticket.token,
                        allowed_methods: account.mfa.get_methods(),
                    }));
                }

                (account, friendly_name)
            } else {
                return Err(Error::InvalidCredentials);
            }
        }
        DataLogin::MFA {
            mfa_ticket,
            mfa_response,
            friendly_name,
        } => {
            // Resolve the MFA ticket
            let ticket = rauth
                .database
                .find_ticket_by_token(&mfa_ticket)
                .await?
                .ok_or(Error::InvalidToken)?;

            // Find the corresponding account
            let mut account = rauth.database.find_account(&ticket.account_id).await?;

            // Verify the MFA response
            account.consume_mfa_response(rauth, mfa_response).await?;
            (account, friendly_name)
        }
    };

    // Generate a session name
    let name = name.unwrap_or_else(|| "Unknown".to_string());

    // Prevent disabled accounts from logging in
    if account.disabled {
        return Err(Error::DisabledAccount);
    }

    // Create and return a new session
    Ok(Json(ResponseLogin::Success(
        account.create_session(rauth, name).await?,
    )))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use iso8601_timestamp::Timestamp;

    use crate::test::*;

    #[async_std::test]
    async fn success() {
        let rauth = for_test("login::success").await;

        Account::new(
            &rauth,
            "example@validemail.com".into(),
            "password_insecure".into(),
            false,
        )
        .await
        .unwrap();

        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::login::login]).await;

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
        let rauth = for_test("login::fail_disabled_account").await;

        let mut account = Account::new(
            &rauth,
            "example@validemail.com".into(),
            "password_insecure".into(),
            false,
        )
        .await
        .unwrap();

        account.disabled = true;
        account.save(&rauth).await.unwrap();

        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::login::login]).await;

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

    #[async_std::test]
    async fn fail_unverified_account() {
        let rauth = for_test("login::fail_unverified_account").await;

        let mut account = Account::new(
            &rauth,
            "example@validemail.com".into(),
            "password_insecure".into(),
            false,
        )
        .await
        .unwrap();

        account.verification = EmailVerification::Pending {
            token: "".to_string(),
            expiry: Timestamp::now_utc(),
        };

        account.save(&rauth).await.unwrap();

        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::login::login]).await;

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

        assert_eq!(res.status(), Status::Forbidden);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"UnverifiedAccount\"}".into())
        );
    }
}
