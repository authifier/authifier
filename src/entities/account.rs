use mongodb::bson::DateTime;
use wither::bson::doc;
use wither::prelude::*;

use super::MFATicket;

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
    token: String,
    expiry: DateTime,
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
    )
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

impl Account {
    pub fn generate_ticket(_method: ()) -> MFATicket {
        // determine if we can generate an MFA ticket
        // return it if we can
        // otherwise throw error

        unimplemented!()
    }
}
