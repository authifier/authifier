use chrono::Duration;
use iso8601_timestamp::Timestamp;

use crate::{
    config::EmailVerificationConfig,
    models::{
        totp::Totp, Account, AuthFlow, DeletionInfo, EmailVerification, MFAMethod, MFAResponse,
        MFATicket, PasswordAuth, PasswordReset, SSOAuth, Session,
    },
    util::{hash_password, normalise_email},
    Authifier, AuthifierEvent, Error, Result, Success,
};

impl Account {
    /// Save model
    pub async fn save(&self, authifier: &Authifier) -> Success {
        authifier.database.save_account(self).await
    }

    /// Create a new account
    pub async fn new(
        authifier: &Authifier,
        email: String,
        plaintext_password: String,
        verify_email: bool,
    ) -> Result<Account> {
        // Hash the user's password
        let password = hash_password(plaintext_password)?;

        // Get a normalised representation of the user's email
        let email_normalised = normalise_email(email.clone());

        // Try to find an existing account
        if let Some(mut account) = authifier
            .database
            .find_account_by_normalised_email(&email_normalised)
            .await?
        {
            // Resend account verification or send password reset
            if let EmailVerification::Pending { .. } = &account.verification {
                account.start_email_verification(authifier).await?;
            } else {
                account.start_password_reset(authifier).await?;
            }

            Ok(account)
        } else {
            // Create a new account
            let mut account = Account {
                id: ulid::Ulid::new().to_string(),

                email,
                email_normalised,

                disabled: false,
                verification: EmailVerification::Verified,
                deletion: None,
                lockout: None,

                auth_flow: AuthFlow::Password(PasswordAuth {
                    password,
                    mfa: Default::default(),
                    password_reset: None,
                }),
            };

            // Send email verification
            if verify_email {
                account.start_email_verification(authifier).await?;
            } else {
                account.save(authifier).await?;
            }

            // Create and push event
            authifier
                .publish_event(AuthifierEvent::CreateAccount {
                    account: account.clone(),
                })
                .await;

            Ok(account)
        }
    }

    /// Create a new account from SSO claims
    pub async fn from_claims(
        authifier: &Authifier,
        idp_id: String,
        sub_id: serde_json::Value,
        email: String,
    ) -> Result<Account> {
        // Create a new account
        let account = Account {
            id: ulid::Ulid::new().to_string(),

            email: email.clone(),
            email_normalised: normalise_email(email),

            disabled: false,
            verification: EmailVerification::Verified,
            deletion: None,
            lockout: None,

            auth_flow: AuthFlow::SSO(SSOAuth { idp_id, sub_id }),
        };

        account.save(authifier).await?;

        // Create and push event
        authifier
            .publish_event(AuthifierEvent::CreateAccount {
                account: account.clone(),
            })
            .await;

        Ok(account)
    }

    /// Create a new session
    pub async fn create_session(&self, authifier: &Authifier, name: String) -> Result<Session> {
        let session = Session {
            id: ulid::Ulid::new().to_string(),
            token: nanoid!(64),

            user_id: self.id.clone(),
            name,

            subscription: None,
        };

        // Save to database
        authifier.database.save_session(&session).await?;

        // Create and push event
        authifier
            .publish_event(AuthifierEvent::CreateSession {
                session: session.clone(),
            })
            .await;

        Ok(session)
    }

    /// Send account verification email
    pub async fn start_email_verification(&mut self, authifier: &Authifier) -> Success {
        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &authifier.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.verify.url, token);

            smtp.send_email(
                self.email.clone(),
                &templates.verify,
                json!({
                    "email": self.email.clone(),
                    "url": url
                }),
            )?;

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

        self.save(authifier).await
    }

    /// Send account verification to new email
    pub async fn start_email_move(&mut self, authifier: &Authifier, new_email: String) -> Success {
        // This method should and will never be called on an unverified account,
        // but just validate this just in case.
        if let EmailVerification::Pending { .. } = self.verification {
            return Err(Error::UnverifiedAccount);
        }

        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &authifier.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.verify.url, token);

            smtp.send_email(
                new_email.clone(),
                &templates.verify,
                json!({
                    "email": self.email.clone(),
                    "url": url
                }),
            )?;

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

        self.save(authifier).await
    }

    /// Send password reset email
    pub async fn start_password_reset(&mut self, authifier: &Authifier) -> Success {
        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &authifier.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.reset.url, token);

            smtp.send_email(
                self.email.clone(),
                &templates.reset,
                json!({
                    "email": self.email.clone(),
                    "url": url
                }),
            )?;

            let AuthFlow::Password(auth) = &mut self.auth_flow else {
                return Ok(());
            };

            auth.password_reset = Some(PasswordReset {
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

        self.save(authifier).await
    }

    /// Begin account deletion process by sending confirmation email
    ///
    /// If email verification is not on, the account will be marked for deletion instantly
    pub async fn start_account_deletion(&mut self, authifier: &Authifier) -> Success {
        if let EmailVerificationConfig::Enabled {
            templates,
            expiry,
            smtp,
        } = &authifier.config.email_verification
        {
            let token = nanoid!(32);
            let url = format!("{}{}", templates.deletion.url, token);

            smtp.send_email(
                self.email.clone(),
                &templates.deletion,
                json!({
                    "email": self.email.clone(),
                    "url": url
                }),
            )?;

            self.deletion = Some(DeletionInfo::WaitingForVerification {
                token,
                expiry: Timestamp::from_unix_timestamp_ms(
                    chrono::Utc::now()
                        .checked_add_signed(Duration::seconds(expiry.expire_password_reset))
                        .expect("failed to checked_add_signed")
                        .timestamp_millis(),
                ),
            });

            self.save(authifier).await
        } else {
            self.schedule_deletion(authifier).await
        }
    }

    /// Verify a user's password is correct
    pub fn verify_password(&self, plaintext_password: &str) -> Success {
        let AuthFlow::Password(auth) = &self.auth_flow else {
            return Ok(());
        };

        argon2::verify_encoded(&auth.password, plaintext_password.as_bytes())
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
    pub async fn consume_mfa_response(
        &mut self,
        authifier: &Authifier,
        response: MFAResponse,
        ticket: Option<MFATicket>,
    ) -> Success {
        let AuthFlow::Password(auth) = &mut self.auth_flow else {
            return Ok(());
        };

        let allowed_methods = auth.mfa.get_methods();

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
                    if let Totp::Enabled { .. } = &auth.mfa.totp_token {
                        // Use TOTP code at generation if applicable
                        if let Some(ticket) = ticket {
                            if let Some(code) = ticket.last_totp_code {
                                if code == totp_code {
                                    return Ok(());
                                }
                            }
                        }

                        // Otherwise read current TOTP token
                        if auth.mfa.totp_token.generate_code()? == totp_code {
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
                    if let Some(index) = auth
                        .mfa
                        .recovery_codes
                        .iter()
                        .position(|x| x == &recovery_code)
                    {
                        auth.mfa.recovery_codes.remove(index);
                        self.save(authifier).await
                    } else {
                        Err(Error::InvalidToken)
                    }
                } else {
                    Err(Error::DisallowedMFAMethod)
                }
            }
        }
    }

    /// Delete all sessions for an account
    pub async fn delete_all_sessions(
        &self,
        authifier: &Authifier,
        exclude_session_id: Option<String>,
    ) -> Success {
        authifier
            .database
            .delete_all_sessions(&self.id, exclude_session_id.clone())
            .await?;

        // Create and push event
        authifier
            .publish_event(AuthifierEvent::DeleteAllSessions {
                user_id: self.id.to_string(),
                exclude_session_id,
            })
            .await;

        Ok(())
    }

    /// Disable an account
    pub async fn disable(&mut self, authifier: &Authifier) -> Success {
        self.disabled = true;
        self.delete_all_sessions(authifier, None).await?;
        self.save(authifier).await
    }

    /// Schedule an account for deletion
    pub async fn schedule_deletion(&mut self, authifier: &Authifier) -> Success {
        self.deletion = Some(DeletionInfo::Scheduled {
            after: Timestamp::from_unix_timestamp_ms(
                chrono::Utc::now()
                    .checked_add_signed(Duration::weeks(1))
                    .expect("failed to checked_add_signed")
                    .timestamp_millis(),
            ),
        });

        self.disable(authifier).await
    }
}
