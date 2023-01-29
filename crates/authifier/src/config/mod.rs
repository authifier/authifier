mod blocklists;
mod captcha;
mod email_verification;
mod ip_resolve;
mod passwords;
mod shield;

pub use blocklists::*;
pub use captcha::*;
pub use email_verification::*;
pub use ip_resolve::*;
pub use passwords::*;
pub use shield::*;

/// Authifier configuration
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

    /// Authifier Shield settings
    pub shield: Shield,

    /// Email verification
    pub email_verification: EmailVerificationConfig,

    /// Whether to only allow registrations if the user has an invite code
    pub invite_only: bool,

    /// Whether this application is running behind Cloudflare
    pub resolve_ip: ResolveIp,
}
