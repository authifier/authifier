/// SMTP mail server configuration
#[derive(Serialize, Deserialize)]
pub struct SMTPSettings {
    /// Sender address
    pub from: String,

    /// Reply-To address
    pub reply_to: Option<String>,

    /// SMTP host
    pub host: String,

    /// SMTP port
    pub port: Option<i32>,

    /// SMTP username
    pub username: String,

    /// SMTP password
    pub password: String,

    /// Whether to use TLS
    pub use_tls: Option<bool>,
}

/// Email template
#[derive(Serialize, Deserialize)]
pub struct Template {
    /// Title of the email
    pub title: String,
    /// Plain text version of this email
    pub text: String,
    /// HTML version of this email
    pub html: Option<String>,
    /// URL to redirect people to from the email
    ///
    /// Use `{{url}}` to fill this field.
    ///
    /// Any given URL will be suffixed with a unique token if applicable.
    ///
    /// e.g. `https://example.com?t=` becomes `https://example.com?t=UNIQUE_CODE`
    pub url: String,
}

/// Email templates
#[derive(Serialize, Deserialize)]
pub struct Templates {
    /// Template for email verification
    pub verify: Template,
    /// Template for password reset
    pub reset: Template,
    /// Template for welcome email
    ///
    /// Unlike the other two, this one isn't required for email verification to function.
    pub welcome: Option<Template>,
}

/// Email expiration config
#[derive(Serialize, Deserialize)]
pub struct EmailExpiryConfig {
    /// How long email verification codes should last for (in seconds)
    pub expire_verification: i64,
    /// How long password reset codes should last for (in seconds)
    pub expire_password_reset: i64,
}

impl Default for EmailExpiryConfig {
    fn default() -> EmailExpiryConfig {
        EmailExpiryConfig {
            expire_verification: 3600 * 24,
            expire_password_reset: 3600,
        }
    }
}

/// Email verification config
#[derive(Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum EmailVerificationConfig {
    /// Don't require email verification
    Disabled,
    /// Use email verification
    Enabled {
        smtp: SMTPSettings,
        templates: Templates,
        expiry: EmailExpiryConfig,
    },
}

impl Default for EmailVerificationConfig {
    fn default() -> EmailVerificationConfig {
        EmailVerificationConfig::Disabled
    }
}
