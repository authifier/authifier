//! Logout of current session
//! POST /session/logout
use rauth::{models::Session, RAuth, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Logout
///
/// Delete current session.
#[openapi(tag = "Session")]
#[post("/logout")]
pub async fn logout(rauth: &State<RAuth>, session: Session) -> Result<EmptyResponse> {
    rauth
        .database
        .delete_session(&session.id)
        .await
        .map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, _) = for_test_authenticated("logout::success").await;
        let client = bootstrap_rocket_with_auth(
            rauth.clone(),
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
            rauth.database.find_session(&session.id).await.unwrap_err(),
            Error::UnknownUser
        );
    }

    #[async_std::test]
    async fn fail_invalid_session() {
        use rocket::http::Header;

        let client = bootstrap_rocket(
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

    #[async_std::test]
    async fn fail_no_session() {
        let client = bootstrap_rocket(
            "logout",
            "fail_no_session",
            routes![crate::routes::session::logout::logout],
        )
        .await;

        let res = client.post("/logout").dispatch().await;

        assert_eq!(res.status(), Status::Unauthorized);
    }
}
