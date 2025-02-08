use iso8601_timestamp::Timestamp;

use super::MultiFactorAuthentication;

/// Email verification status
#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordReset {
    /// Token required to change password
    pub token: String,
    /// Time at which this token expires
    pub expiry: Timestamp,
}

/// Account deletion information
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "status")]
pub enum DeletionInfo {
    /// The user must confirm deletion by email
    WaitingForVerification { token: String, expiry: Timestamp },
    /// The account is scheduled for deletion
    Scheduled { after: Timestamp },
    /// This account was deleted
    Deleted,
}

/// Lockout information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lockout {
    /// Attempt counter
    pub attempts: i32,
    /// Time at which this lockout expires
    pub expiry: Option<Timestamp>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordAuth {
    /// Argon2 hashed password
    pub password: String,

    /// Multi-factor authentication information
    pub mfa: MultiFactorAuthentication,

    /// Password reset information
    pub password_reset: Option<PasswordReset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SSOAuth {
    /// Auth Provider
    pub idp_id: String,

    /// Subject ID
    pub sub_id: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuthFlow {
    Password(PasswordAuth),
    SSO(SSOAuth),
}

/// Account model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// User's email
    pub email: String,

    /// Normalised email
    ///
    /// (see https://github.com/insertish/authifier/#how-does-authifier-work)
    pub email_normalised: String,

    /// Whether the account is disabled
    #[serde(default)]
    pub disabled: bool,

    /// Email verification status
    pub verification: EmailVerification,

    /// Account deletion information
    pub deletion: Option<DeletionInfo>,

    /// Account lockout
    pub lockout: Option<Lockout>,

    /// Authentication flow
    pub auth_flow: AuthFlow,
}
