use chrono::Duration;

pub struct SMTP {
    pub from: String,
    pub host: String,
    pub username: String,
    pub password: String,
}

pub enum EmailVerification {
    Disabled,
    Enabled {
        success_redirect_uri: String,
        verification_expiry: Duration,

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
