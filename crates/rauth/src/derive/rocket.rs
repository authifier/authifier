use rocket::{
    http::{ContentType, Status},
    outcome::Outcome,
    request::{self, FromRequest},
    response::{self, Responder},
    Request, Response,
};

use crate::{
    models::{Account, MFATicket, Session, UnvalidatedTicket, ValidatedTicket},
    Error, RAuth,
};

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
            Error::UnverifiedAccount => Status::Forbidden,
            Error::UnknownUser => Status::NotFound,
            Error::EmailFailed => Status::InternalServerError,
            Error::InvalidCredentials => Status::Unauthorized,
            Error::InvalidToken => Status::Unauthorized,
            Error::MissingInvite => Status::BadRequest,
            Error::InvalidInvite => Status::BadRequest,
            Error::CompromisedPassword => Status::BadRequest,
            Error::DisabledAccount => Status::Unauthorized,
            Error::ShortPassword => Status::BadRequest,
            Error::Blacklisted => {
                // Fail blacklisted email addresses.
                const RESP: &str = "{\"type\":\"DisallowedContactSupport\", \"email\":\"support@revolt.chat\", \"note\":\"If you see this messages right here, you're probably doing something you shouldn't be.\"}";

                return Response::build()
                    .status(Status::Unauthorized)
                    .sized_body(RESP.len(), std::io::Cursor::new(RESP))
                    .ok();
            }
            Error::TotpAlreadyEnabled => Status::BadRequest,
            Error::DisallowedMFAMethod => Status::BadRequest,
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = Error;

    #[allow(clippy::collapsible_match)]
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let header_session_token = request
            .headers()
            .get("x-session-token")
            .next()
            .map(|x| x.to_string());

        match (request.rocket().state::<RAuth>(), header_session_token) {
            (Some(rauth), Some(token)) => {
                if let Ok(session) = rauth.database.find_session_by_token(&token).await {
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Account {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.guard::<Session>().await {
            Outcome::Success(session) => {
                let rauth = request.rocket().state::<RAuth>().unwrap();

                if let Ok(account) = rauth.database.find_account(&session.user_id).await {
                    Outcome::Success(account)
                } else {
                    Outcome::Failure((Status::InternalServerError, Error::InternalError))
                }
            }
            Outcome::Forward(_) => unreachable!(),
            Outcome::Failure(err) => Outcome::Failure(err),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MFATicket {
    type Error = Error;

    #[allow(clippy::collapsible_match)]
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let header_mfa_ticket = request
            .headers()
            .get("x-mfa-ticket")
            .next()
            .map(|x| x.to_string());

        match (request.rocket().state::<RAuth>(), header_mfa_ticket) {
            (Some(rauth), Some(token)) => {
                if let Ok(ticket) = rauth.database.find_ticket_by_token(&token).await {
                    if let Some(ticket) = ticket {
                        Outcome::Success(ticket)
                    } else {
                        Outcome::Failure((Status::Unauthorized, Error::InvalidToken))
                    }
                } else {
                    Outcome::Failure((Status::Unauthorized, Error::InternalError))
                }
            }
            (_, _) => Outcome::Failure((Status::Unauthorized, Error::MissingHeaders)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ValidatedTicket {
    type Error = Error;

    #[allow(clippy::collapsible_match)]
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.guard::<MFATicket>().await {
            Outcome::Success(ticket) => {
                if ticket.validated {
                    let rauth = request
                        .rocket()
                        .state::<RAuth>()
                        .expect("This code is unreachable.");

                    if ticket.claim(rauth).await.is_ok() {
                        Outcome::Success(ValidatedTicket(ticket))
                    } else {
                        Outcome::Failure((Status::InternalServerError, Error::InternalError))
                    }
                } else {
                    Outcome::Failure((Status::Forbidden, Error::InvalidToken))
                }
            }
            Outcome::Forward(f) => Outcome::Forward(f),
            Outcome::Failure(err) => Outcome::Failure(err),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UnvalidatedTicket {
    type Error = Error;

    #[allow(clippy::collapsible_match)]
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.guard::<MFATicket>().await {
            Outcome::Success(ticket) => {
                if !ticket.validated {
                    Outcome::Success(UnvalidatedTicket(ticket))
                } else {
                    Outcome::Failure((Status::Forbidden, Error::InvalidToken))
                }
            }
            Outcome::Forward(f) => Outcome::Forward(f),
            Outcome::Failure(err) => Outcome::Failure(err),
        }
    }
}
