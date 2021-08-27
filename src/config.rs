#[derive(Serialize, Deserialize)]
pub enum PasswordScanning {
    /// Disable password scanning
    None,
    /// Use a custom password list
    Custom { passwords: Vec<String> },
    /// Block the top 100k passwords from HIBP
    Top100k,
    /// Use the Have I Been Pwned? API
    HIBP { api_key: String },
}

impl Default for PasswordScanning {
    fn default() -> PasswordScanning {
        PasswordScanning::Top100k
    }
}

#[derive(Serialize, Deserialize)]
pub enum EmailBlockList {
    /// Don't block any emails
    Disabled,
    /// Block a custom list of domains
    Custom { domains: Vec<String> },
    /// Disposable mail list maintained by revolt.chat
    RevoltSourceList,
}

impl Default for EmailBlockList {
    fn default() -> EmailBlockList {
        EmailBlockList::RevoltSourceList
    }
}

#[derive(Serialize, Deserialize)]
pub enum Captcha {
    /// Don't require captcha verification
    Disabled,
    /// Use hCaptcha to validate sensitive requests
    HCaptcha { secret: String },
}

impl Default for Captcha {
    fn default() -> Captcha {
        Captcha::Disabled
    }
}

#[derive(Serialize, Deserialize)]
pub struct SMTPSettings {
    pub from: String,
    pub reply_to: Option<String>,
    pub host: String,
    pub username: String,
    pub password: String,
}

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
    pub url: Option<String>,
}

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

#[derive(Serialize, Deserialize)]
pub struct EmailExpiry {
    /// How long email verification codes should last for (in seconds)
    expire_verification: i64,
    /// How long password reset codes should last for (in seconds)
    expire_password_reset: i64,
}

impl Default for EmailExpiry {
    fn default() -> EmailExpiry {
        EmailExpiry {
            expire_verification: 3600 * 24,
            expire_password_reset: 3600,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum EmailVerification {
    /// Don't require email verification
    Disabled,
    /// Use email verification
    Enabled {
        smtp: SMTPSettings,
        templates: Templates,
        expiry: EmailExpiry,
    },
}

impl Default for EmailVerification {
    fn default() -> EmailVerification {
        EmailVerification::Disabled
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// Check if passwords are compromised
    pub password_scanning: PasswordScanning,

    /// Email block list
    ///
    /// Use to block common disposable mail providers.
    /// Enabled by default.
    pub email_block_list: EmailBlockList,

    /// Ping the SMTP server to verify the inbox exists
    ///
    /// This could fail in some cases, but could help users
    pub verify_email_existence: bool,

    /// Captcha options
    pub captcha: Captcha,

    /// Email verification
    pub email_verification: EmailVerification,

    /// Whether to only allow registrations if the user has an invite code
    pub invite_only: bool,
}
