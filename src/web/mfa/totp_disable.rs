/// Generate a new secret for TOTP.
/// DELETE /mfa/totp
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Result};

#[delete("/totp")]
pub async fn totp_disable(
    auth: &State<Auth>,
    mut account: Account,
) -> Result<EmptyResponse> {
    account.mfa.totp_token = Totp::Disabled;

    account.save_to_db(&auth.db).await.map(|_| EmptyResponse)
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("totp_disable::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::web::mfa::totp_enable::totp_enable,
                crate::web::mfa::totp_disable::totp_disable
            ],
        )
        .await;

        let res = client
            .delete("/totp")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
