use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;

use mongodb::bson::DateTime;
use wither::bson::doc;
use wither::prelude::*;

use crate::logic::Auth;
use crate::util::{Error, Result};

use super::Session;

fn is_false(t: &bool) -> bool {
    !t
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AccountVerification {
    Verified,
    Pending {
        token: String,
        expiry: DateTime,
    },
    Moving {
        new_email: String,
        token: String,
        expiry: DateTime,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordReset {
    pub token: String,
    pub expiry: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Totp {
    Disabled,
    Pending { secret: String },
    Enabled { secret: String },
}

impl Totp {
    pub fn is_disabled(&self) -> bool {
        if let Totp::Disabled = self {
            true
        } else {
            false
        }
    }
}

impl Default for Totp {
    fn default() -> Totp {
        Totp::Disabled
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MultiFactorAuthentication {
    /// Allow password-less email OTP login
    /// (1-Factor)
    #[serde(skip_serializing_if = "is_false", default)]
    pub enable_email_otp: bool,

    /// Allow trusted handover
    /// (1-Factor)
    #[serde(skip_serializing_if = "is_false", default)]
    pub enable_trusted_handover: bool,

    /// Allow email MFA
    /// (2-Factor)
    #[serde(skip_serializing_if = "is_false", default)]
    pub enable_email_mfa: bool,

    /// TOTP MFA token, enabled if present
    /// (2-Factor)
    #[serde(skip_serializing_if = "Totp::is_disabled", default)]
    pub totp_token: Totp,

    /// Security Key MFA token, enabled if present
    /// (2-Factor)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_key_token: Option<String>,

    /// Recovery codes
    pub recovery_codes: Vec<String>,
}

impl MultiFactorAuthentication {
    pub fn is_2fa_enabled(&self) -> bool {
        self.enable_email_mfa || !self.totp_token.is_disabled() || self.security_key_token.is_some()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiFactorStatus {
    email_otp: bool,
    trusted_handover: bool,
    email_mfa: bool,
    totp_mfa: bool,
    security_key_mfa: bool,
    recovery_active: bool,
}

impl From<MultiFactorAuthentication> for MultiFactorStatus {
    fn from(item: MultiFactorAuthentication) -> Self {
        MultiFactorStatus {
            email_otp: item.enable_email_otp,
            trusted_handover: item.enable_trusted_handover,
            email_mfa: item.enable_email_mfa,
            totp_mfa: !item.totp_token.is_disabled(),
            security_key_mfa: item.security_key_token.is_some(),
            recovery_active: !item.recovery_codes.is_empty(),
        }
    }
}

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(
    collection_name = "accounts",
    index(
        keys = r#"doc!{"email_normalised": 1}"#,
        options = r#"doc!{"unique": true}"#
    ),
    index(keys = r#"doc!{"email": 1}"#, options = r#"doc!{"unique": true}"#)
)]
pub struct Account {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub email: String,
    pub email_normalised: String,
    pub password: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    pub verification: AccountVerification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset: Option<PasswordReset>,

    pub mfa: MultiFactorAuthentication,
}

#[derive(Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
}

impl From<Account> for AccountInfo {
    fn from(item: Account) -> Self {
        AccountInfo {
            id: item.id.expect("`id` present"),
            email: item.email,
        }
    }
}

impl Account {
    pub async fn save_to_db(&mut self, db: &mongodb::Database) -> Result<()> {
        self.save(&db, None)
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "save",
                with: "account",
            })
    }

    pub fn verify_password(&self, password: &str) -> Result<()> {
        argon2::verify_encoded(&self.password, password.as_bytes())
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
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Account {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.guard::<Session>().await {
            Outcome::Success(session) => {
                let auth = request.rocket().state::<Auth>().unwrap();

                if let Ok(Some(account)) =
                    Account::find_one(&auth.db, doc! { "_id": session.user_id }, None).await
                {
                    Outcome::Success(account)
                } else {
                    Outcome::Failure((Status::InternalServerError, Error::InvalidSession))
                }
            }
            Outcome::Forward(_) => unreachable!(),
            Outcome::Failure(err) => Outcome::Failure(err),
        }
    }
}
