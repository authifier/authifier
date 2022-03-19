/// Revoke an active session
/// DELETE /session/:id
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Error, Result};

/// # Revoke Session
/// 
/// Delete a specific active session.
#[openapi(tag = "Session")]
#[delete("/<id>")]
pub async fn revoke(auth: &State<Auth>, session: Session, id: String) -> Result<EmptyResponse> {
    Session::delete_many(
        &auth.db,
        doc! {
            "_id": id,
            "user_id": session.user_id
        },
        None,
    )
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

        let (db, auth, session, _) = for_test_authenticated("revoke::success").await;
        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::revoke::revoke]).await;

        let res = client
            .delete(format!("/{}", session.id.as_ref().unwrap()))
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert!(
            Session::find_one(&db, doc! { "_id": session.id.unwrap() }, None)
                .await
                .unwrap()
                .is_none()
        );
    }
}
