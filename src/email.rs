use handlebars::Handlebars;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde_json::Value;

use crate::options::{Template, SMTP};
use crate::util::{Error, Result};

lazy_static! {
    static ref HANDLEBARS: Handlebars<'static> = Handlebars::new();
}

pub fn send(smtp: &SMTP, message: Message) -> Result<()> {
    SmtpTransport::relay(&smtp.host)
        .unwrap()
        .credentials(Credentials::new(
            smtp.username.clone(),
            smtp.password.clone(),
        ))
        .build()
        .send(&message)
        .map_err(|_| Error::EmailFailed)?;

    Ok(())
}

pub fn generate_multipart(text: String, html: Option<String>) -> MultiPart {
    if let Some(html) = html {
        MultiPart::mixed().multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header("text/plain; charset=utf8".parse::<header::ContentType>().unwrap())
                        .body(text.to_string()),
                )
                .multipart(
                    MultiPart::related().singlepart(
                        SinglePart::builder()
                            .header("text/html; charset=utf8".parse::<header::ContentType>().unwrap())
                            .body(html.to_string()),
                    ),
                )
            )
    } else {
        MultiPart::mixed()
            .singlepart(
            SinglePart::builder()
                .header("text/html; charset=utf8".parse::<header::ContentType>().unwrap())
                .body(text.to_string()),
        )
    }
}

impl Template {
    pub fn generate_email(&self, from: &str, to: &str, variables: Value) -> Result<Message> {
        Message::builder()
            .from(from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(self.title)
            .multipart(generate_multipart(
                HANDLEBARS
                    .render_template(self.text, &variables)
                    .map_err(|_| Error::RenderFail)?.to_string(),
                if let Some(html) = self.html { Some(HANDLEBARS
                    .render_template(html, &variables)
                    .map_err(|_| Error::RenderFail)?.to_string()) } else { None },
            ))
            .map_err(|_| Error::InternalError)
    }
}
