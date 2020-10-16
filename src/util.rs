use snafu::Snafu;
use std::io::Cursor;
use rocket::request::Request;
use rocket::http::ContentType;
use validator::ValidationErrors;
use rocket::response::{self, Response, Responder};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to validate fields."))]
    FailedValidation {
        error: ValidationErrors
    },
    /*#[snafu(display("Email is invalid!"))]
    InvalidEmail,
    #[snafu(display("Username is invalid!"))]
    InvalidUsername,
    #[snafu(display("Password is invalid!"))]
    InvalidPassword*/
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let string = format!("{:?}", self);
        Response::build()
            .sized_body(string.len(), Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .ok()
    }
}
