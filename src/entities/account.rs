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

fn is_false(t: &bool) -> bool {
    !t
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MultiFactorAuthentication {
    /// Allow password-less email OTP login
    #[serde(skip_serializing_if = "is_false", default)]
    enable_email_otp: bool,

    /// Allow trusted handover
    #[serde(skip_serializing_if = "is_false", default)]
    enable_trusted_handover: bool,

    /// Allow email MFA
    #[serde(skip_serializing_if = "is_false", default)]
    enable_email_mfa: bool,

    /// TOTP MFA token, enabled if present
    #[serde(skip_serializing_if = "Option::is_none")]
    totp_token: Option<String>,

    /// Security Key MFA token, enabled if present
    #[serde(skip_serializing_if = "Option::is_none")]
    security_key_token: Option<String>,

    /// Recovery codes
    recovery_codes: Vec<String>,
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
