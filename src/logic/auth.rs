use std::collections::{HashMap, HashSet};

use lettre::transport::smtp::authentication::Credentials;
use lettre::SmtpTransport;
use mongodb::Database;
use nanoid::nanoid;

use crate::config::{
    Captcha, Config, EmailBlockList, EmailVerification, PasswordScanning, Template,
};
use crate::entities::*;
use crate::util::{self, Error, Result};

lazy_static! {
    static ref ARGON_CONFIG: argon2::Config<'static> = argon2::Config::default();
    static ref HANDLEBARS: handlebars::Handlebars<'static> = handlebars::Handlebars::new();
}

pub struct Auth {
    pub db: Database,
    pub config: Config,

    pub smtp_transport: Option<SmtpTransport>,
    pub compromised_passwords: HashSet<String>,
    pub email_block_list: Option<HashSet<String>>,
}

impl Auth {
    pub fn new(db: Database, config: Config) -> Auth {
        if config.verify_email_existence {
            unimplemented!() // TODO: MX + SMTP validation
        }

        Auth {
            db,
            smtp_transport: match &config.email_verification {
                EmailVerification::Enabled { smtp, .. } => Some(
                    SmtpTransport::relay(&smtp.host)
                        .unwrap()
                        .credentials(Credentials::new(
                            smtp.username.clone(),
                            smtp.password.clone(),
                        ))
                        .build(),
                ),
                EmailVerification::Disabled => None,
            },
            compromised_passwords: match &config.password_scanning {
                PasswordScanning::None | PasswordScanning::HIBP { .. } => HashSet::new(),
                PasswordScanning::Custom { passwords } => passwords.into_iter().cloned().collect(),
                PasswordScanning::Top100k => include_str!("../../assets/pwned100k.txt")
                    .split('\n')
                    .skip(4)
                    .map(|x| x.into())
                    .collect(),
            },
            email_block_list: match &config.email_block_list {
                EmailBlockList::Disabled => None,
                EmailBlockList::Custom { domains } => Some(domains.into_iter().cloned().collect()),
                EmailBlockList::RevoltSourceList => Some(
                    include_str!("../../assets/revolt_source_list.txt")
                        .split('\n')
                        .map(|x| x.into())
                        .collect(),
                ),
            },
            config,
        }
    }

    // #region Validation
    /// Consume a Captcha verification token
    pub async fn check_captcha(&self, token: Option<String>) -> Result<()> {
        match &self.config.captcha {
            Captcha::HCaptcha { secret } => {
                if let Some(token) = token {
                    let mut map = HashMap::new();
                    map.insert("secret", secret.clone());
                    map.insert("response", token);

                    let client = reqwest::Client::new();
                    if let Ok(response) = client
                        .post("https://hcaptcha.com/siteverify")
                        .form(&map)
                        .send()
                        .await
                    {
                        #[derive(Serialize, Deserialize)]
                        struct CaptchaResponse {
                            success: bool,
                        }

                        let result: CaptchaResponse =
                            response.json().await.map_err(|_| Error::CaptchaFailed)?;

                        if result.success {
                            Ok(())
                        } else {
                            Err(Error::CaptchaFailed)
                        }
                    } else {
                        Err(Error::CaptchaFailed)
                    }
                } else {
                    Err(Error::CaptchaFailed)
                }
            }
            Captcha::Disabled => Ok(()),
        }
    }

    /// Consume an invite
    pub async fn check_invite(&self, invite: Option<String>) -> Result<Option<Invite>> {
        if self.config.invite_only {
            if let Some(invite) = invite {
                if let Some(invite) = Invite::find_one(
                    &self.db,
                    doc! {
                        "_id": invite
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "find_one",
                    with: "invite",
                })? {
                    return Ok(Some(invite));
                }
            }

            return Err(Error::InvalidInvite);
        }

        Ok(None)
    }

    /// Check that an email is valid
    pub async fn validate_email(&self, email: &str) -> Result<()> {
        if !validator::validate_email(email) {
            return Err(Error::IncorrectData { with: "email" });
        }

        // Check if the email is blacklisted
        if let Some(list) = &self.email_block_list {
            if let Some(domain) = email.split('@').last() {
                if list.contains(domain) {
                    return Err(Error::Blacklisted);
                }
            }
        }

        if self.config.verify_email_existence {
            // TODO: implement
        }

        Ok(())
    }

    /// Check whether a password can be used
    pub async fn validate_password(&self, password: &str) -> Result<()> {
        if self.compromised_passwords.contains(password) {
            Err(Error::CompromisedPassword)
        } else {
            Ok(())
        }
    }
    // #endregion

    // #region Operations
    /// Create a new account without validating fields.
    ///
    /// You should NOT handle errors from this function unless
    /// if you're debugging this library, it can open you up to
    /// potential attacks such as email enumeration. Although,
    /// for something like an admin panel, that's fine.
    pub async fn create_account(
        &self,
        email: String,
        plaintext_password: String,
        verify_email: bool,
    ) -> Result<Account> {
        // Get a normalised representation of the user's email.
        // This is also a unique index on the account collection
        // so we don't actually have to check if it already exists
        // in the database, just let it fail.
        let email_normalised = util::normalise_email(email.clone());

        // Hash the user's password.
        let password = argon2::hash_encoded(
            plaintext_password.as_bytes(),
            nanoid!(24).as_bytes(),
            &ARGON_CONFIG,
        )
        .map_err(|_| Error::InternalError)?;

        // Send email verification.
        let verification = if verify_email {
            if let EmailVerification::Enabled { templates, .. } = self.config.email_verification {
                unimplemented!()
            } else {
                AccountVerification::Verified
            }
        } else {
            AccountVerification::Verified
        };

        // Construct new Account.
        let mut account = Account {
            id: None,

            email,
            email_normalised,
            password,

            disabled: None,
            verification,
            password_reset: None,
            mfa: MultiFactorAuthentication::default(),
        };

        // Commit to database.
        account
            .save(&self.db, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "save",
                with: "account",
            })?;

        Ok(account)
    }
    // #endregion

    // #region Email Utilities
    pub fn send_email(
        &self,
        to: String,
        template: Template,
        variables: handlebars::JsonValue,
    ) -> Result<()> {
        if let Some(sender) = &self.smtp_transport {
            if let EmailVerification::Enabled { smtp, .. } = &self.config.email_verification {
                let m = lettre::Message::builder()
                    .from(smtp.from.parse().map_err(|_| Error::EmailFailed)?)
                    .to(to.parse().map_err(|_| Error::EmailFailed)?)
                    .subject(template.title);

                let m = if let Some(reply_to) = &smtp.reply_to {
                    m.reply_to(reply_to.parse().map_err(|_| Error::EmailFailed)?)
                } else {
                    m
                };

                let text = self.render_template(&template.text, &variables)?;
                let m = if let Some(html) = template.html {
                    m.multipart(lettre::message::MultiPart::alternative_plain_html(
                        text,
                        self.render_template(&html, &variables)?,
                    ))
                } else {
                    m.body(text)
                }
                .map_err(|_| Error::EmailFailed)?;

                use lettre::Transport;
                sender.send(&m).map_err(|_| Error::EmailFailed)?;

                Ok(())
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    pub fn render_template(&self, text: &str, variables: &handlebars::JsonValue) -> Result<String> {
        Ok(HANDLEBARS
            .render_template(text, variables)
            .map_err(|_| Error::RenderFail)?
            .to_string())
    }
    // #endregion
}
