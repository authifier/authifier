//! Revoke an active session
//! DELETE /session/:id
use rauth::{models::Session, Error, RAuth, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Revoke Session
///
/// Delete a specific active session.
#[openapi(tag = "Session")]
#[delete("/<id>")]
pub async fn revoke(rauth: &State<RAuth>, user: Session, id: String) -> Result<EmptyResponse> {
    let session = rauth.database.find_session(&id).await?;

    if session.user_id != user.user_id {
        return Err(Error::InvalidToken);
    }

    rauth
        .database
        .delete_session(&id)
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

        let (rauth, session, _) = for_test_authenticated("revoke::success").await;
        let client = bootstrap_rocket_with_auth(
            rauth.clone(),
            routes![crate::routes::session::revoke::revoke],
        )
        .await;

        let res = client
            .delete(format!("/{}", session.id))
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert_eq!(
            rauth.database.find_session(&session.id).await.unwrap_err(),
            Error::UnknownUser
        );
    }
}
