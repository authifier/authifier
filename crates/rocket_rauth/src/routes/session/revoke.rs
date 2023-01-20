//! Revoke an active session
//! DELETE /session/:id
use authifier::{models::Session, Authifier, Error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Revoke Session
///
/// Delete a specific active session.
#[openapi(tag = "Session")]
#[delete("/<id>")]
pub async fn revoke(
    authifier: &State<Authifier>,
    user: Session,
    id: String,
) -> Result<EmptyResponse> {
    let session = authifier.database.find_session(&id).await?;

    if session.user_id != user.user_id {
        return Err(Error::InvalidToken);
    }

    session.delete(authifier).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (authifier, session, _, _) = for_test_authenticated("revoke::success").await;
        let client = bootstrap_rocket_with_auth(
            authifier.clone(),
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
            authifier
                .database
                .find_session(&session.id)
                .await
                .unwrap_err(),
            Error::UnknownUser
        );
    }
}
