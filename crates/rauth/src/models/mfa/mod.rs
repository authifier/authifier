use self::totp::Totp;

pub mod totp;

/// Whether a boolean is false
// fn is_false(t: &bool) -> bool {
//     !t
// }

/// MFA configuration
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MultiFactorAuthentication {
    /// Allow password-less email OTP login
    /// (1-Factor)
    // #[serde(skip_serializing_if = "is_false", default)]
    // pub enable_email_otp: bool,

    /// Allow trusted handover
    /// (1-Factor)
    // #[serde(skip_serializing_if = "is_false", default)]
    // pub enable_trusted_handover: bool,

    /// Allow email MFA
    /// (2-Factor)
    // #[serde(skip_serializing_if = "is_false", default)]
    // pub enable_email_mfa: bool,

    /// TOTP MFA token, enabled if present
    /// (2-Factor)
    #[serde(skip_serializing_if = "Totp::is_disabled", default)]
    pub totp_token: Totp,

    /// Security Key MFA token, enabled if present
    /// (2-Factor)
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub security_key_token: Option<String>,

    /// Recovery codes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub recovery_codes: Vec<String>,
}
