mod blocklists;
mod captcha;
mod email_verification;
mod passwords;

pub use blocklists::*;
pub use captcha::*;
pub use email_verification::*;
pub use passwords::*;

/// rAuth configuration
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Check if passwords are compromised
    pub password_scanning: PasswordScanning,

    /// Email block list
    ///
    /// Use to block common disposable mail providers.
    /// Enabled by default.
    pub email_block_list: EmailBlockList,

    /// Captcha options
    pub captcha: Captcha,

    /// Email verification
    pub email_verification: EmailVerificationConfig,

    /// Whether to only allow registrations if the user has an invite code
    pub invite_only: bool,
}
