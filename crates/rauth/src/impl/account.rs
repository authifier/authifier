use chrono::Duration;
use iso8601_timestamp::Timestamp;

use crate::{
    config::EmailVerificationConfig,
    models::{
        totp::Totp, Account, DeletionInfo, EmailVerification, MFAMethod, MFAResponse,
        PasswordReset, Session,
    },
    util::{hash_password, normalise_email},
    Error, RAuth, Result, Success,
};

impl Account {
    /// Save model
    pub async fn save(&self, rauth: &RAuth) -> Success {
        rauth.database.save_account(self).await
    }

    /// Create a new account
    pub async fn new(
        rauth: &RAuth,
        email: String,
        plaintext_password: String,
        verify_email: bool,
    ) -> Result<Account> {
        // Hash the user's password
        let password = hash_password(plaintext_password)?;

        // Get a normalised representation of the user's email
        let email_normalised = normalise_email(email.clone());

        // Try to find an existing account
        if let Some(mut account) = rauth
            .database
            .find_account_by_normalised_email(&email_normalised)
            .await?
        {
            // Resend account verification or send password reset
            if let EmailVerification::Pending { .. } = &account.verification {
                account.start_email_verification(rauth).await?;
            } else {
                account.start_password_reset(rauth).await?;
            }

            Ok(account)
        } else {
            // Create a new account
            let mut account = Account {
                id: ulid::Ulid::new().to_string(),

                email,
                email_normalised,
                password,

                disabled: false,
                verification: EmailVerification::Verified,
                password_reset: None,
                deletion: None,

                mfa: Default::default(),
            };

            // Send email verification
            if verify_email {
                account.start_email_verification(rauth).await?;
            } else {
                account.save(rauth).await?;
            }

            Ok(account)
        }
    }

    /// Create a new session
    pub async fn create_session(&self, rauth: &RAuth, name: String) -> Result<Session> {
        let session = Session {
            id: ulid::Ulid::new().to_string(),
            token: nanoid!(64),

            user_id: self.id.clone(),
            name,

            subscription: None,
        };

        rauth.database.save_session(&session).await?;

        Ok(session)
    }

    /// Send account verification email
    pub async fn start_email_verification(&mut self, rauth: &RAuth) -> Success {
        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &rauth.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.verify.url, token);

            smtp.send_email(self.email.clone(), &templates.verify, json!({ "url": url }))
                .ok();

            self.verification = EmailVerification::Pending {
                token,
                expiry: Timestamp::from_unix_timestamp_ms(
                    chrono::Utc::now()
                        .checked_add_signed(Duration::seconds(expiry.expire_verification))
                        .expect("failed to checked_add_signed")
                        .timestamp_millis(),
                ),
            };
        } else {
            self.verification = EmailVerification::Verified;
        }

        self.save(rauth).await
    }

    /// Send account verification to new email
    pub async fn start_email_move(&mut self, rauth: &RAuth, new_email: String) -> Success {
        // This method should and will never be called on an unverified account,
        // but just validate this just in case.
        if let EmailVerification::Pending { .. } = self.verification {
            return Err(Error::UnverifiedAccount);
        }

        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &rauth.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.verify.url, token);

            smtp.send_email(new_email.clone(), &templates.verify, json!({ "url": url }))
                .ok();

            self.verification = EmailVerification::Moving {
                new_email,
                token,
                expiry: Timestamp::from_unix_timestamp_ms(
                    chrono::Utc::now()
                        .checked_add_signed(Duration::seconds(expiry.expire_verification))
                        .expect("failed to checked_add_signed")
                        .timestamp_millis(),
                ),
            };
        } else {
            self.email_normalised = normalise_email(new_email.clone());
            self.email = new_email;
        }

        self.save(rauth).await
    }

    /// Send password reset email
    pub async fn start_password_reset(&mut self, rauth: &RAuth) -> Success {
        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &rauth.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.reset.url, token);

            smtp.send_email(self.email.clone(), &templates.reset, json!({ "url": url }))
                .ok();

            self.password_reset = Some(PasswordReset {
                token,
                expiry: Timestamp::from_unix_timestamp_ms(
                    chrono::Utc::now()
                        .checked_add_signed(Duration::seconds(expiry.expire_password_reset))
                        .expect("failed to checked_add_signed")
                        .timestamp_millis(),
                ),
            });
        } else {
            return Err(Error::OperationFailed);
        }

        self.save(rauth).await
    }

    /// Begin account deletion process by sending confirmation email
    ///
    /// If email verification is not on, the account will be marked for deletion instantly
    pub async fn start_account_deletion(&mut self, rauth: &RAuth) -> Success {
        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &rauth.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.deletion.url, token);

            smtp.send_email(
                self.email.clone(),
                &templates.deletion,
                json!({ "url": url }),
            )
            .ok();

            self.deletion = Some(DeletionInfo::WaitingForVerification {
                token,
                expiry: Timestamp::from_unix_timestamp_ms(
                    chrono::Utc::now()
                        .checked_add_signed(Duration::seconds(expiry.expire_password_reset))
                        .expect("failed to checked_add_signed")
                        .timestamp_millis(),
                ),
            });

            self.save(rauth).await
        } else {
            self.schedule_deletion(rauth).await
        }
    }

    /// Verify a user's password is correct
    pub fn verify_password(&self, plaintext_password: &str) -> Success {
        argon2::verify_encoded(&self.password, plaintext_password.as_bytes())
            .map(|v| {
                if v {
                    Ok(())
                } else {
                    Err(Error::InvalidCredentials)
                }
            })
            // To prevent user enumeration, we should ignore
            // the error and pretend the password is wrong.
            .map_err(|_| Error::InvalidCredentials)?
    }

    /// Validate an MFA response
    pub async fn consume_mfa_response(&mut self, rauth: &RAuth, response: MFAResponse) -> Success {
        let allowed_methods = self.mfa.get_methods();

        match response {
            MFAResponse::Password { password } => {
                if allowed_methods.contains(&MFAMethod::Password) {
                    self.verify_password(&password)
                } else {
                    Err(Error::DisallowedMFAMethod)
                }
            }
            MFAResponse::Totp { totp_code } => {
                if allowed_methods.contains(&MFAMethod::Totp) {
                    if let Totp::Enabled { .. } = &self.mfa.totp_token {
                        if self.mfa.totp_token.generate_code()? == totp_code {
                            Ok(())
                        } else {
                            Err(Error::InvalidToken)
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    Err(Error::DisallowedMFAMethod)
                }
            }
            MFAResponse::Recovery { recovery_code } => {
                if allowed_methods.contains(&MFAMethod::Recovery) {
                    if let Some(index) = self
                        .mfa
                        .recovery_codes
                        .iter()
                        .position(|x| x == &recovery_code)
                    {
                        self.mfa.recovery_codes.remove(index);
                        self.save(rauth).await
                    } else {
                        Err(Error::InvalidToken)
                    }
                } else {
                    Err(Error::DisallowedMFAMethod)
                }
            }
        }
    }

    /// Disable an account
    pub async fn disable(&mut self, rauth: &RAuth) -> Success {
        self.disabled = true;
        rauth.database.delete_all_sessions(&self.id, None).await?;
        self.save(rauth).await
    }

    /// Schedule an account for deletion
    pub async fn schedule_deletion(&mut self, rauth: &RAuth) -> Success {
        self.deletion = Some(DeletionInfo::Scheduled {
            after: Timestamp::from_unix_timestamp_ms(
                chrono::Utc::now()
                    .checked_add_signed(Duration::weeks(1))
                    .expect("failed to checked_add_signed")
                    .timestamp_millis(),
            ),
        });

        self.disable(rauth).await
    }
}
