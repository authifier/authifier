use iso8601_timestamp::Timestamp;

use super::MultiFactorAuthentication;

/// Whether a boolean is false
fn is_false(t: &bool) -> bool {
    !t
}

/// Email verification status
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum EmailVerification {
    /// Account is verified
    Verified,
    /// Pending email verification
    Pending { token: String, expiry: Timestamp },
    /// Moving to a new email
    Moving {
        new_email: String,
        token: String,
        expiry: Timestamp,
    },
}

/// Password reset information
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordReset {
    /// Token required to change password
    pub token: String,
    /// Time at which this token expires
    pub expiry: Timestamp,
}

/// Account deletion information
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DeletionInfo {
    /// The user must confirm deletion by email
    WaitingForVerification { token: String, expiry: Timestamp },
    /// The account is scheduled for deletion
    Scheduled { after: Timestamp },
    /// This account was deleted
    Deleted
}

/// Account model
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// User's email
    pub email: String,

    /// Normalised email
    ///
    /// (see https://github.com/insertish/rauth/#how-does-rauth-work)
    pub email_normalised: String,

    /// Argon2 hashed password
    pub password: String,

    /// Whether the account is disabled
    #[serde(skip_serializing_if = "is_false", default)]
    pub disabled: bool,

    /// Email verification status
    pub verification: EmailVerification,

    /// Password reset information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset: Option<PasswordReset>,

    /// Account deletion information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion: Option<DeletionInfo>,

    /// Multi-factor authentication information
    pub mfa: MultiFactorAuthentication,
}
