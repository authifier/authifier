use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::util::{Error, Result};
use crate::options::SMTP;

pub fn send(smtp: &SMTP, message: Message) -> Result<()> {
    SmtpTransport::relay(&smtp.host)
        .unwrap()
        .credentials(Credentials::new(
            smtp.username.clone(),
            smtp.password.clone()
        ))
        .build()
        .send(&message)
        .map_err(|_| Error::EmailFailed)?;

    Ok(())
}

pub fn generate_multipart(text: &str, html: &str) -> MultiPart {
    MultiPart::mixed().multipart(
        MultiPart::alternative()
            .singlepart(
                SinglePart::quoted_printable()
                    .header(header::ContentType(
                        "text/plain; charset=utf8".parse().unwrap(),
                    ))
                    .body(text),
            )
            .multipart(
                MultiPart::related().singlepart(
                    SinglePart::eight_bit()
                        .header(header::ContentType(
                            "text/html; charset=utf8".parse().unwrap(),
                        ))
                        .body(html),
                ),
            ),
    )
}
