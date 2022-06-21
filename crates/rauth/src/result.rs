#[derive(Serialize, Debug, JsonSchema, PartialEq)]
#[serde(tag = "type")]
pub enum Error {
    IncorrectData {
        with: &'static str,
    },
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

    EmailFailed,
    InvalidToken,
    MissingInvite,
    InvalidInvite,
    InvalidCredentials,

    CompromisedPassword,
    DisabledAccount,
    ShortPassword,
    Blacklisted,

    TotpAlreadyEnabled,
    DisallowedMFAMethod,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Success = Result<()>;
