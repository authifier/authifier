//! Login to an account
//! POST /session/login
use std::ops::Add;
use std::time::Duration;

use iso8601_timestamp::Timestamp;
use rauth::models::{EmailVerification, Lockout, MFAMethod, MFAResponse, MFATicket, Session};
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
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
    MFA {
        /// Unvalidated or authorised MFA ticket
        ///
        /// Used to resolve the correct account
        mfa_ticket: String,
        /// Valid MFA response
        ///
        /// This will take precedence over the `password` field where applicable
        mfa_response: Option<MFAResponse>,
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
            friendly_name,
        } => {
            // Try to find the account we want
            let email_normalised = normalise_email(email);

            // Lookup the email in database
            if let Some(mut account) = rauth
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

                // Check for account lockout
                if let Some(lockout) = &account.lockout {
                    if let Some(expiry) = lockout.expiry {
                        if expiry.to_unix_timestamp_ms()
                            > Timestamp::now_utc().to_unix_timestamp_ms()
                        {
                            return Err(Error::LockedOut);
                        }
                    }
                }

                // Verify the password is correct.
                if let Err(err) = account.verify_password(&password) {
                    // Lock out account if attempts are too high
                    if let Some(lockout) = &mut account.lockout {
                        lockout.attempts += 1;

                        // Allow 3 attempts
                        //
                        // Lockout for 1 minute on 3rd attempt
                        // Lockout for 5 minutes on 4th attempt
                        // Lockout for 1 hour on each subsequent attempt
                        if lockout.attempts >= 3 {
                            lockout.expiry = Some(Timestamp::now_utc().add(Duration::from_secs(
                                if lockout.attempts >= 5 {
                                    3600
                                } else if lockout.attempts == 4 {
                                    300
                                } else {
                                    60
                                },
                            )));
                        }
                    } else {
                        account.lockout = Some(Lockout {
                            attempts: 1,
                            expiry: None,
                        });
                    }

                    account.save(rauth).await?;
                    return Err(err);
                }

                // Clear lockout information if present
                if account.lockout.is_some() {
                    account.lockout = None;
                    account.save(rauth).await?;
                }

                // Check whether an MFA step is required
                if account.mfa.is_active() {
                    // Create a new ticket
                    let mut ticket = MFATicket::new(account.id, false);
                    ticket.populate(&account.mfa).await;
                    ticket.save(rauth).await?;

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
            if let Some(mfa_response) = mfa_response {
                account
                    .consume_mfa_response(rauth, mfa_response, Some(ticket))
                    .await?;
            } else if !ticket.authorised {
                return Err(Error::InvalidToken);
            }

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

    use super::ResponseLogin;

    #[async_std::test]
    async fn success() {
        let (rauth, receiver) = for_test("login::success").await;

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

        let event = receiver.try_recv().expect("an event");
        if !matches!(event, RAuthEvent::CreateSession { .. }) {
            panic!("Received incorrect event type. {:?}", event);
        }
    }

    #[async_std::test]
    async fn success_totp_mfa() {
        let (rauth, _, mut account, _) =
            for_test_authenticated("create_ticket::success_totp_mfa").await;

        let totp = Totp::Enabled {
            secret: "secret".to_string(),
        };

        account.mfa.totp_token = totp.clone();
        account.save(&rauth).await.unwrap();

        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::login::login]).await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "email@revolt.chat",
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        let response = serde_json::from_str::<crate::routes::session::login::ResponseLogin>(
            &res.into_string().await.unwrap(),
        )
        .expect("`ResponseLogin`");

        if let ResponseLogin::MFA {
            ticket,
            allowed_methods,
        } = response
        {
            assert!(allowed_methods.contains(&MFAMethod::Totp));

            let res = client
                .post("/login")
                .header(ContentType::JSON)
                .body(
                    json!({
                        "mfa_ticket": ticket,
                        "mfa_response": {
                            "totp_code": totp.generate_code().expect("totp code")
                        }
                    })
                    .to_string(),
                )
                .dispatch()
                .await;

            assert_eq!(res.status(), Status::Ok);
            assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
        } else {
            panic!("expected `ResponseLogin::MFA`")
        }
    }

    #[async_std::test]
    async fn success_totp_stored_mfa() {
        let (rauth, _, mut account, _) =
            for_test_authenticated("create_ticket::success_totp_stored_mfa").await;

        let totp = Totp::Enabled {
            secret: "secret".to_string(),
        };

        account.mfa.totp_token = totp.clone();
        account.save(&rauth).await.unwrap();

        let mut ticket = MFATicket::new(account.id.to_string(), true);
        ticket.last_totp_code = Some("token from earlier".into());
        ticket.save(&rauth).await.unwrap();

        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::login::login]).await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "mfa_ticket": ticket.token,
                    "mfa_response": {
                        "totp_code": "token from earlier"
                    }
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }

    #[async_std::test]
    async fn fail_totp_invalid_mfa() {
        let (rauth, _, mut account, _) =
            for_test_authenticated("create_ticket::fail_totp_invalid_mfa").await;

        let totp = Totp::Enabled {
            secret: "secret".to_string(),
        };

        account.mfa.totp_token = totp.clone();
        account.save(&rauth).await.unwrap();

        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::login::login]).await;

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "email@revolt.chat",
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        let response = serde_json::from_str::<crate::routes::session::login::ResponseLogin>(
            &res.into_string().await.unwrap(),
        )
        .expect("`ResponseLogin`");

        if let ResponseLogin::MFA {
            ticket,
            allowed_methods,
        } = response
        {
            assert!(allowed_methods.contains(&MFAMethod::Totp));

            let res = client
                .post("/login")
                .header(ContentType::JSON)
                .body(
                    json!({
                        "mfa_ticket": ticket,
                        "mfa_response": {
                            "totp_code": "some random data"
                        }
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
        } else {
            panic!("expected `ResponseLogin::MFA`")
        }
    }

    #[async_std::test]
    async fn fail_invalid_user() {
        let (client, _) = bootstrap_rocket(
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
        let (rauth, _) = for_test("login::fail_disabled_account").await;

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
        let (rauth, _) = for_test("login::fail_unverified_account").await;

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

    #[async_std::test]
    async fn fail_locked_account() {
        let (rauth, _) = for_test("login::fail_locked_account").await;

        let mut account = Account::new(
            &rauth,
            "example@validemail.com".into(),
            "password_insecure".into(),
            false,
        )
        .await
        .unwrap();

        account.save(&rauth).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            rauth.clone(),
            routes![crate::routes::session::login::login],
        )
        .await;

        // Attempt 1
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "wrong_password"
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

        // Attempt 2
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "wrong_password"
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

        // Attempt 3
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "example@validemail.com",
                    "password": "wrong_password"
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

        // Attempt 4: Locked Out
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
            Some("{\"type\":\"LockedOut\"}".into())
        );

        // Pretend it expired
        account.lockout = Some(Lockout {
            attempts: 9001,
            expiry: Some(Timestamp::now_utc()),
        });

        account.save(&rauth).await.unwrap();

        // Once it expires, we can log in.
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

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }
}
