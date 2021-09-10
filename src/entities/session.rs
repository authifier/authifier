use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;

use wither::bson::doc;
use wither::prelude::*;

use crate::logic::Auth;
use crate::util::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebPushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Model, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
