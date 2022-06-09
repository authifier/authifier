use okapi::openapi3::{self, SecurityScheme, SecuritySchemeData};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
    response::OpenApiResponderInner,
};

use crate::{
    models::{Account, MFATicket, Session, UnvalidatedTicket, ValidatedTicket},
    Error,
};

impl OpenApiResponderInner for Error {
    fn responses(
        gen: &mut OpenApiGenerator,
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

        definitions.insert(
            "RAuth Error".to_string(),
            schemars::schema::Schema::Object(schema.schema),
        );

        content.insert(
            "application/json".to_string(),
            openapi3::MediaType {
                schema: Some(schemars::schema::SchemaObject {
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

macro_rules! from_request {
    ($struct_name:ident, $name:expr, $header_name:expr, $description:expr) => {
        impl<'r> OpenApiFromRequest<'r> for $struct_name {
            fn from_request_input(
                _gen: &mut OpenApiGenerator,
                _name: String,
                _required: bool,
            ) -> rocket_okapi::Result<RequestHeaderInput> {
                let mut requirements = schemars::Map::new();
                requirements.insert($name.to_owned(), vec![]);

                Ok(RequestHeaderInput::Security(
                    $name.to_owned(),
                    SecurityScheme {
                        data: SecuritySchemeData::ApiKey {
                            name: $header_name.to_owned(),
                            location: "header".to_owned(),
                        },
                        description: Some($description.to_owned()),
                        extensions: schemars::Map::new(),
                    },
                    requirements,
                ))
            }
        }
    };
}

from_request!(
    Session,
    "Session Token",
    "x-session-token",
    "Used to authenticate as a user."
);

from_request!(
    Account,
    "Session Token",
    "x-session-token",
    "Used to authenticate as a user."
);

from_request!(
    MFATicket,
    "MFA Ticket",
    "x-mfa-ticket",
    "Used to authorise a request."
);

from_request!(
    ValidatedTicket,
    "Valid MFA Ticket",
    "x-mfa-ticket",
    "Used to authorise a request."
);

from_request!(
    UnvalidatedTicket,
    "Unvalidated MFA Ticket",
    "x-mfa-ticket",
    "Used to authorise a request."
);
