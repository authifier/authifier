use lettre::{
    transport::smtp::{authentication::Credentials, client::Tls},
    SmtpTransport,
};

use crate::{Error, Result, Success};

/// SMTP mail server configuration
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
pub struct Templates {
    /// Template for email verification
    pub verify: Template,
    /// Template for password reset
    pub reset: Template,
    /// Template for account deletion
    pub deletion: Template,
    /// Template for welcome email
    ///
    /// Unlike the other two, this one isn't required for email verification to function.
    pub welcome: Option<Template>,
}

/// Email expiration config
#[derive(Serialize, Deserialize, Clone)]
pub struct EmailExpiryConfig {
    /// How long email verification codes should last for (in seconds)
    pub expire_verification: i64,
    /// How long password reset codes should last for (in seconds)
    pub expire_password_reset: i64,
    /// How long account deletion codes should last for (in seconds)
    pub expire_account_deletion: i64,
}

impl Default for EmailExpiryConfig {
    fn default() -> EmailExpiryConfig {
        EmailExpiryConfig {
            expire_verification: 3600 * 24,
            expire_password_reset: 3600,
            expire_account_deletion: 3600,
        }
    }
}

/// Email verification config
#[derive(Serialize, Deserialize, Clone)]
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

impl SMTPSettings {
    /// Create SMTP transport
    pub fn create_transport(&self) -> SmtpTransport {
        let relay = SmtpTransport::relay(&self.host).unwrap();
        let relay = if let Some(port) = self.port {
            relay.port(port.try_into().unwrap())
        } else {
            relay
        };

        let relay = if let Some(false) = self.use_tls {
            relay.tls(Tls::None)
        } else {
            relay
        };

        relay
            .credentials(Credentials::new(
                self.username.clone(),
                self.password.clone(),
            ))
            .build()
    }

    /// Render an email template
    fn render_template(text: &str, variables: &handlebars::JsonValue) -> Result<String> {
        lazy_static! {
            static ref HANDLEBARS: handlebars::Handlebars<'static> = handlebars::Handlebars::new();
        }

        HANDLEBARS
            .render_template(text, variables)
            .map_err(|_| Error::RenderFail)
    }

    /// Send an email
    pub fn send_email(
        &self,
        address: String,
        template: &Template,
        variables: handlebars::JsonValue,
    ) -> Success {
        let m = lettre::Message::builder()
            .from(self.from.parse().expect("valid `smtp_from`"))
            .to(address.parse().expect("valid `smtp_to`"))
            .subject(template.title.clone());

        let m = if let Some(reply_to) = &self.reply_to {
            m.reply_to(reply_to.parse().expect("valid `smtp_reply_to`"))
        } else {
            m
        };

        let text =
            SMTPSettings::render_template(&template.text, &variables).expect("valid `template`");

        let m = if let Some(html) = &template.html {
            m.multipart(lettre::message::MultiPart::alternative_plain_html(
                text,
                SMTPSettings::render_template(html, &variables).expect("valid `template`"),
            ))
        } else {
            m.body(text)
        }
        .expect("valid `message`");

        use lettre::Transport;
        let sender = self.create_transport();
        if let Err(error) = sender.send(&m) {
            error!(
                "Failed to send email to {}!\nlettre error: {}",
                address, error
            );

            return Err(Error::EmailFailed);
        }

        Ok(())
    }
}
