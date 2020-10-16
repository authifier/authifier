use json;
use snafu::Snafu;
use std::io::Cursor;
use serde::Serialize;
use rocket::request::Request;
use rocket::http::ContentType;
use validator::ValidationErrors;
use rocket::response::{self, Response, Responder};

#[derive(Serialize, Debug, Snafu)]
#[serde(tag = "type")]
pub enum Error {
    #[snafu(display("Failed to validate fields."))]
    #[serde(rename = "failed_validation")]
    FailedValidation {
        error: ValidationErrors
    },
    #[snafu(display("Encountered a database error."))]
    #[serde(rename = "database_error")]
    DatabaseError,
    #[snafu(display("This user does not exist!"))]
    #[serde(rename = "unknown_user")]
    UnknownUser,
    #[snafu(display("Wrong password."))]
    #[serde(rename = "wrong_password")]
    WrongPassword,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let string = json!(self).to_string();
        Response::build()
            .sized_body(string.len(), Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .ok()
    }
}
