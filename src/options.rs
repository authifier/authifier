use chrono::Duration;

pub struct SMTP {
    pub from: String,
    pub host: String,
    pub username: String,
    pub password: String,
}

pub struct Template {
    pub title: &'static str,
    pub text: &'static str,
    pub html: &'static str,
}

pub struct Templates {
    pub verify_email: Template,
    pub reset_password: Template,
    pub welcome: Option<Template>,
}

pub enum EmailVerification {
    Disabled,
    Enabled {
        welcome_redirect_uri: String,
        success_redirect_uri: String,
        password_reset_url: Option<String>,
        
        verification_expiry: Duration,
        password_reset_expiry: Duration,

        templates: Templates,
        smtp: SMTP,
    },
}

pub struct Options {
    pub email_verification: EmailVerification,
    pub base_url: String,
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

impl Options {
    pub fn new() -> Options {
        Options {
            email_verification: EmailVerification::Disabled,
            base_url: "https://example.com".to_string(),
        }
    }

    pub fn email_verification(self, email_verification: EmailVerification) -> Options {
        Options {
            email_verification,
            ..self
        }
    }

    pub fn base_url(self, base_url: String) -> Options {
        Options { base_url, ..self }
    }
}
