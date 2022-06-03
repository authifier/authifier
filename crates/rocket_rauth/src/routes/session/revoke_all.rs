//! Revoke all sessions
//! DELETE /session/all
use rauth::{models::Session, RAuth, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete All Sessions
///
/// Delete all active sessions, optionally including current one.
#[openapi(tag = "Session")]
#[delete("/all?<revoke_self>")]
pub async fn revoke_all(
    rauth: &State<RAuth>,
    session: Session,
    revoke_self: Option<bool>,
) -> Result<EmptyResponse> {
    let ignore = if revoke_self.unwrap_or(false) {
        None
    } else {
        Some(session.id)
    };

    rauth
        .database
        .delete_all_sessions(&session.user_id, ignore)
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

        let (rauth, session, account) = for_test_authenticated("revoke_all::success").await;

        for i in 1..=3 {
            account
                .create_session(&rauth, format!("session{}", i))
                .await
                .unwrap();
        }

        let client = bootstrap_rocket_with_auth(
            rauth.clone(),
            routes![crate::routes::session::revoke_all::revoke_all],
        )
        .await;

        let res = client
            .delete("/all?revoke_self=true")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert!(rauth
            .database
            .find_sessions(&session.user_id)
            .await
            .unwrap()
            .is_empty());
    }

    #[async_std::test]
    async fn success_not_including_self() {
        use rocket::http::Header;

        let (rauth, session, account) =
            for_test_authenticated("revoke_all::success_not_including_self").await;

        for i in 1..=3 {
            account
                .create_session(&rauth, format!("session{}", i))
                .await
                .unwrap();
        }

        let client = bootstrap_rocket_with_auth(
            rauth.clone(),
            routes![crate::routes::session::revoke_all::revoke_all],
        )
        .await;

        let res = client
            .delete("/all?revoke_self=false")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert_eq!(
            rauth
                .database
                .find_sessions(&session.user_id)
                .await
                .unwrap()
                .len(),
            1
        );
    }
}
