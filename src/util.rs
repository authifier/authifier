use regex::Regex;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde::Serialize;
use snafu::Snafu;
use std::io::Cursor;
use validator::ValidationErrors;

#[derive(Serialize, Debug, Snafu)]
#[serde(tag = "type")]
pub enum Error {
    #[snafu(display("Failed to validate fields."))]
    FailedValidation { error: ValidationErrors },
    #[snafu(display("Encountered a database error."))]
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    #[snafu(display("Encountered an internal error."))]
    InternalError,
    #[snafu(display("Operation did not succeed."))]
    OperationFailed,
    #[snafu(display("Failed to render Email template."))]
    RenderFail,
    #[snafu(display("Missing authentication headers."))]
    MissingHeaders,
    #[snafu(display("Invalid session information."))]
    InvalidSession,
    #[snafu(display("User account has not been verified."))]
    UnverifiedAccount,
    #[snafu(display("This user does not exist!"))]
    UnknownUser,
    #[snafu(display("Email is use."))]
    EmailInUse,
    #[snafu(display("Email failed to send."))]
    EmailFailed,
    #[snafu(display("Email or password is incorrect."))]
    InvalidCredentials,
    #[snafu(display("This token is not valid."))]
    InvalidToken,
    #[snafu(display("This instance is invite-only, provide invite parameter."))]
    MissingInvite,
    #[snafu(display("This invite does not exist."))]
    InvalidInvite,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// HTTP response builder for Error enum
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Serialize the error data structure into JSON.
        let string = json!(self).to_string();

        // Build and send the request.
        Response::build()
            .sized_body(string.len(), Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .status(Status::UnprocessableEntity)
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
