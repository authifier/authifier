use snafu::Snafu;
use validator::ValidationErrors;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to validate struct."))]
    FailedValidation {
        error: ValidationErrors
    },
    #[snafu(display("Email is invalid!"))]
    InvalidEmail,
    #[snafu(display("Username is invalid!"))]
    InvalidUsername,
    #[snafu(display("Password is invalid!"))]
    InvalidPassword
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
