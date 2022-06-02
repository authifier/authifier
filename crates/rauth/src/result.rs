#[cfg(feature = "rocket_impl")]
use rocket::{
    http::ContentType,
    http::Status,
    response::{self, Responder},
    serde::json::json,
    Request, Response,
};

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
    Blacklisted,

    TotpAlreadyEnabled,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// HTTP response builder for Error enum
#[cfg(feature = "rocket_impl")]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            Error::IncorrectData { .. } => Status::BadRequest,
            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::InternalError => Status::InternalServerError,
            Error::OperationFailed => Status::InternalServerError,
            Error::RenderFail => Status::InternalServerError,
            Error::MissingHeaders => Status::BadRequest,
            Error::CaptchaFailed => Status::BadRequest,
            Error::InvalidSession => Status::Unauthorized,
            Error::UnverifiedAccount => Status::BadRequest,
            Error::UnknownUser => Status::NotFound,
            Error::EmailFailed => Status::InternalServerError,
            Error::InvalidCredentials => Status::Unauthorized,
            Error::InvalidToken => Status::Unauthorized,
            Error::MissingInvite => Status::BadRequest,
            Error::InvalidInvite => Status::BadRequest,
            Error::CompromisedPassword => Status::BadRequest,
            Error::DisabledAccount => Status::Unauthorized,
            Error::Blacklisted => {
                // Fail blacklisted email addresses.
                const RESP: &str = "{\"type\":\"DisallowedContactSupport\", \"email\":\"support@revolt.chat\", \"note\":\"If you see this messages right here, you're probably doing something you shouldn't be.\"}";

                return Response::build()
                    .status(Status::Unauthorized)
                    .sized_body(RESP.len(), std::io::Cursor::new(RESP))
                    .ok();
            }
            Error::TotpAlreadyEnabled => Status::BadRequest,
        };

        // Serialize the error data structure into JSON.
        let string = json!(self).to_string();

        // Build and send the request.
        Response::build()
            .sized_body(string.len(), std::io::Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .status(status)
            .ok()
    }
}

#[cfg(feature = "okapi_impl")]
impl rocket_okapi::response::OpenApiResponderInner for Error {
    fn responses(
        gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> std::result::Result<okapi::openapi3::Responses, rocket_okapi::OpenApiError> {
        let mut content = okapi::Map::new();

        let settings = schemars::gen::SchemaSettings::default().with(|s| {
            s.option_nullable = true;
            s.option_add_null_type = false;
            s.definitions_path = "#/components/schemas/".to_string();
        });

        let mut schema_generator = settings.into_generator();
        let schema = schema_generator.root_schema_for::<Error>();

        let definitions = gen.schema_generator().definitions_mut();
        for (key, value) in schema.definitions {
            definitions.insert(key, value);
        }

        definitions.insert(
            "Error".to_string(),
            schemars::schema::Schema::Object(schema.schema),
        );

        content.insert(
            "application/json".to_string(),
            okapi::openapi3::MediaType {
                schema: Some(schemars::schema::SchemaObject {
                    reference: Some("#/components/schemas/Error".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        Ok(okapi::openapi3::Responses {
            default: Some(okapi::openapi3::RefOr::Object(okapi::openapi3::Response {
                content,
                description: "An error occurred.".to_string(),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
