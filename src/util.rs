use okapi::openapi3::{RefOr, SchemaObject};
use regex::Regex;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket_okapi::okapi::openapi3;
use schemars::schema::Schema;
use serde::Serialize;
use serde_json::json;
use std::io::Cursor;

#[derive(Serialize, Debug, JsonSchema)]
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
                    .sized_body(RESP.len(), Cursor::new(RESP))
                    .ok();
            }
            Error::TotpAlreadyEnabled => Status::BadRequest,
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
pub struct EmptyResponse;

impl<'r> Responder<'r, 'static> for EmptyResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(rocket::http::Status { code: 204 })
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

impl rocket_okapi::response::OpenApiResponderInner for Error {
    fn responses(
        gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> std::result::Result<openapi3::Responses, rocket_okapi::OpenApiError> {
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

        definitions.insert("Error".to_string(), Schema::Object(schema.schema));

        content.insert(
            "application/json".to_string(),
            openapi3::MediaType {
                schema: Some(SchemaObject {
                    reference: Some("#/components/schemas/Error".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        Ok(openapi3::Responses {
            default: Some(openapi3::RefOr::Object(openapi3::Response {
                content,
                description: "An error occurred.".to_string(),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

impl rocket_okapi::response::OpenApiResponderInner for EmptyResponse {
    fn responses(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> std::result::Result<openapi3::Responses, rocket_okapi::OpenApiError> {
        let mut responses = okapi::Map::new();

        responses.insert(
            "204".to_string(),
            RefOr::Object(openapi3::Response {
                description: "Success".to_string(),
                ..Default::default()
            }),
        );

        Ok(openapi3::Responses {
            responses,
            ..Default::default()
        })
    }
}
