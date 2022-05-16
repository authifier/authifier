/// Revoke all sessions
/// DELETE /session/all
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Error, Result};

/// # Delete All Sessions
///
/// Delete all active sessions, optionally including current one.
#[openapi(tag = "Session")]
#[delete("/all?<revoke_self>")]
pub async fn revoke_all(
    auth: &State<Auth>,
    session: Session,
    revoke_self: Option<bool>,
) -> Result<EmptyResponse> {
    let revoke_self = revoke_self.unwrap_or(false);
    let mut update = doc! {
        "user_id": session.user_id
    };

    if !revoke_self {
        update.insert(
            "_id",
            doc! {
                "$ne": session.id.unwrap()
            },
        );
    }

    Session::delete_many(&auth.db, update, None)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "delete",
            with: "session",
        })
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
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

    #[cfg(feature = "async-std-runtime")]
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
