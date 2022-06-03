use iso8601_timestamp::Timestamp;

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

/// Time-based one-time password configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Totp {
    /// Disabled
    Disabled,
    /// Waiting for user activation
    Pending { secret: String },
    /// Required on account
    Enabled { secret: String },
}

/// MFA configuration
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

    /// Multi-factor authentication information
    pub mfa: MultiFactorAuthentication,
}
