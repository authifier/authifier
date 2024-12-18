#[derive(Serialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
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
    BlockedByShield,

    InvalidSession,
    UnverifiedAccount,
    UnknownUser,

    EmailFailed,
    InvalidToken,
    InvalidState,
    MissingInvite,
    InvalidInvite,
    InvalidCredentials,

    CompromisedPassword,
    ShortPassword,
    Blacklisted,
    LockedOut,

    TotpAlreadyEnabled,
    DisallowedMFAMethod,

    NotAvailable,

    InvalidEndpoints,
    StateMismatch,

    RequestFailed,
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,

    ContentTypeMismatch,
    InsufficientScope,
    InvalidUserinfo,

    InvalidRedirectUri,
    InvalidIdpId,
    InvalidIdpConfig,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Success = Result<()>;
