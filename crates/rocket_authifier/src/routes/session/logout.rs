//! Logout of current session
//! POST /session/logout
use authifier::{models::Session, Authifier, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Logout
///
/// Delete current session.
#[openapi(tag = "Session")]
#[post("/logout")]
pub async fn logout(authifier: &State<Authifier>, session: Session) -> Result<EmptyResponse> {
    session.delete(authifier).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[tokio::test]
    async fn success() {
        use rocket::http::Header;

        let (authifier, session, _, mut receiver) = for_test_authenticated("logout::success").await;
        let client = bootstrap_rocket_with_auth(
            authifier.clone(),
            routes![crate::routes::session::logout::logout],
        )
        .await;

        let res = client
            .post("/logout")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert_eq!(
            authifier
                .database
                .find_session(&session.id)
                .await
                .unwrap_err(),
            Error::UnknownUser
        );

        let event = receiver.try_recv().expect("an event");
        if let AuthifierEvent::DeleteSession {
            user_id,
            session_id,
        } = event
        {
            assert_eq!(user_id, session.user_id);
            assert_eq!(session_id, session.id);
        } else {
            panic!("Received incorrect event type. {:?}", event);
        }
    }

    #[tokio::test]
    async fn fail_invalid_session() {
        use rocket::http::Header;

        let (client, _) = bootstrap_rocket(
            "logout",
            "fail_invalid_session",
            routes![crate::routes::session::logout::logout],
        )
        .await;

        let res = client
            .post("/logout")
            .header(Header::new("X-Session-Token", "invalid"))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
    }

    #[tokio::test]
    async fn fail_no_session() {
        let (client, _) = bootstrap_rocket(
            "logout",
            "fail_no_session",
            routes![crate::routes::session::logout::logout],
        )
        .await;

        let res = client.post("/logout").dispatch().await;

        assert_eq!(res.status(), Status::Unauthorized);
    }
}
