use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::options::SMTP;

fn send(smtp: &SMTP, message: Message) -> Result<(), String> {
    SmtpTransport::relay(&smtp.host)
        .unwrap()
        .credentials(Credentials::new(
            smtp.username.clone(),
            smtp.password.clone()
        ))
        .build()
        .send(&message)
        .map_err(|err| format!("Failed to send email! {}", err.to_string()))?;

    Ok(())
}

fn generate_multipart(text: &str, html: &str) -> MultiPart {
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
