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
#[cfg(feature = "TODO")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (db, auth, session, account) = for_test_authenticated("revoke_all::success").await;

        for i in 1..=3 {
            auth.create_session(&account, format!("session{}", i))
                .await
                .unwrap();
        }

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::revoke_all::revoke_all])
                .await;

        let res = client
            .delete("/all?revoke_self=true")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert!(
            Session::find_one(&db, doc! { "user_id": session.user_id }, None)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[async_std::test]
    async fn success_not_including_self() {
        use rocket::http::Header;

        let (db, auth, session, account) =
            for_test_authenticated("revoke_all::success_not_including_self").await;

        for i in 1..=3 {
            auth.create_session(&account, format!("session{}", i))
                .await
                .unwrap();
        }

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::revoke_all::revoke_all])
                .await;

        let res = client
            .delete("/all?revoke_self=false")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        assert!(Session::find_one(
            &db,
            doc! { "_id": { "$ne": session.id.as_ref().unwrap() }, "user_id": session.user_id },
            None
        )
        .await
        .unwrap()
        .is_none());

        assert!(
            Session::find_one(&db, doc! { "_id": session.id.as_ref().unwrap() }, None)
                .await
                .unwrap()
                .is_some()
        );
    }
}
