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

        let (db, auth, session, account) = for_test_authenticated("totp_disable::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::web::mfa::totp_generate_secret::totp_generate_secret,
                crate::web::mfa::totp_disable::totp_disable
            ],
        )
        .await;

        let res = client
            .post("/totp")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        let account = Account::find_one(&db, doc! { "_id": account.id.unwrap() }, None)
            .await
            .unwrap()
            .unwrap();

        assert_ne!(account.mfa.totp_token, Totp::Disabled);

        let res = client
            .delete("/totp")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let account = Account::find_one(&db, doc! { "_id": account.id.unwrap() }, None)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(account.mfa.totp_token, Totp::Disabled);
    }
}
