mod blocklists;
mod captcha;
mod email_verification;
mod ip_resolve;
mod passwords;
mod shield;
mod sso;

pub use blocklists::*;
pub use captcha::*;
pub use email_verification::*;
pub use ip_resolve::*;
pub use passwords::*;
use reqwest::Url;
pub use shield::*;
pub use sso::*;

/// Authifier configuration
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Check if passwords are compromised
    #[serde(default)]
    pub password_scanning: PasswordScanning,

    /// Email block list
    ///
    /// Use to block common disposable mail providers.
    /// Enabled by default.
    #[serde(default)]
    pub email_block_list: EmailBlockList,

    /// Captcha options
    #[serde(default)]
    pub captcha: Captcha,

    /// Authifier Shield settings
    #[serde(default)]
    pub shield: Shield,

    /// Email verification
    #[serde(default)]
    pub email_verification: EmailVerificationConfig,

    /// Whether to only allow registrations if the user has an invite code
    #[serde(default)]
    pub invite_only: bool,

    /// Whether this application is running behind Cloudflare
    #[serde(default)]
    pub resolve_ip: ResolveIp,

    /// Single sign-on
    #[serde(default)]
    pub sso: SSO,

    /// Public server URL
    #[serde(default)]
    pub server_url: Option<Url>,
}
