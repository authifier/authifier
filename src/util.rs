use regex::Regex;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde::Serialize;
use std::io::Cursor;
use validator::ValidationErrors;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Error {
    FailedValidation { error: ValidationErrors },
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    InternalError,
    OperationFailed,
    RenderFail,
    MissingHeaders,
    CaptchaFailed,
    InvalidSession,
    UnverifiedAccount,
    UnknownUser,
    EmailInUse,
    EmailFailed,
    InvalidCredentials,
    InvalidToken,
    MissingInvite,
    InvalidInvite,
    CompromisedPassword
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// HTTP response builder for Error enum
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            Error::FailedValidation { .. } => Status::BadRequest,
            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::InternalError => Status::InternalServerError,
            Error::OperationFailed => Status::InternalServerError,
            Error::RenderFail => Status::InternalServerError,
            Error::MissingHeaders => Status::BadRequest,
            Error::CaptchaFailed => Status::BadRequest,
            Error::InvalidSession => Status::Forbidden,
            Error::UnverifiedAccount => Status::BadRequest,
            Error::UnknownUser => Status::NotFound,
            Error::EmailInUse => Status::Conflict,
            Error::EmailFailed => Status::InternalServerError,
            Error::InvalidCredentials => Status::Forbidden,
            Error::InvalidToken => Status::Forbidden,
            Error::MissingInvite => Status::BadRequest,
            Error::InvalidInvite => Status::BadRequest,
            Error::CompromisedPassword => Status::BadRequest,
        };

        // Serialize the error data structure into JSON.
        let string = json!(self).to_string();

        // Build and send the request.
        Response::build()
            .sized_body(string.len(), Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .status(status)
            .ok()
    }
}

pub fn normalise_email(original: String) -> String {
    lazy_static! {
        static ref SPLIT: Regex = Regex::new("([^@]+)(@.+)").unwrap();
        static ref SYMBOL_RE: Regex = Regex::new("\\+.+|\\.").unwrap();
    }

    let split = SPLIT.captures(&original).unwrap();
    let mut clean = SYMBOL_RE
        .replace_all(split.get(1).unwrap().as_str(), "")
        .to_string();
    clean.push_str(split.get(2).unwrap().as_str());

    clean
}
