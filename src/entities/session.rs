use okapi::openapi3::{SecurityScheme, SecuritySchemeData};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;

use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use wither::bson::doc;
use wither::prelude::*;

use crate::logic::Auth;
use crate::util::Error;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebPushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Model, Serialize, Deserialize, JsonSchema)]
#[model(
    collection_name = "sessions",
    index(keys = r#"doc!{"token": 1}"#, options = r#"doc!{"unique": true}"#),
    index(keys = r#"doc!{"user_id": 1}"#)
)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub user_id: String,
    pub token: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription: Option<WebPushSubscription>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SessionInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl From<Session> for SessionInfo {
    fn from(item: Session) -> Self {
        SessionInfo {
            id: item.id.expect("`id` present"),
            name: item.name,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let header_session_token = request
            .headers()
            .get("x-session-token")
            .next()
            .map(|x| x.to_string());

        match (request.rocket().state::<Auth>(), header_session_token) {
            (Some(auth), Some(token)) => {
                if let Ok(session) = Session::find_one(
                    &auth.db,
                    doc! {
                        "token": token
                    },
                    None,
                )
                .await
                {
                    if let Some(session) = session {
                        Outcome::Success(session)
                    } else {
                        Outcome::Failure((Status::Unauthorized, Error::InvalidSession))
                    }
                } else {
                    Outcome::Failure((Status::Unauthorized, Error::InvalidSession))
                }
            }
            (_, _) => Outcome::Failure((Status::Unauthorized, Error::MissingHeaders)),
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for Session {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let mut requirements = schemars::Map::new();
        requirements.insert("Api Key".to_owned(), vec![]);

        Ok(RequestHeaderInput::Security(
            "Api Key".to_owned(),
            SecurityScheme {
                data: SecuritySchemeData::ApiKey {
                    name: "x-session-token".to_owned(),
                    location: "header".to_owned(),
                },
                description: Some("Session Token".to_owned()),
                extensions: schemars::Map::new(),
            },
            requirements,
        ))
    }
}
