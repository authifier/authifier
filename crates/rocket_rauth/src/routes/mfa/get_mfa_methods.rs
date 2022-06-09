//! Fetch available MFA methods.
//! GET /mfa/methods
use rauth::models::{Account, MFAMethod};
use rocket::serde::json::Json;

/// # Get MFA Methods
///
/// Fetch available MFA methods.
#[openapi(tag = "MFA")]
#[get("/methods")]
pub async fn get_mfa_methods(account: Account) -> Json<Vec<MFAMethod>> {
    Json(account.mfa.get_methods())
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use rauth::models::totp::Totp;

    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, _) = for_test_authenticated("get_mfa_methods::success").await;
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::mfa::get_mfa_methods::get_mfa_methods],
        )
        .await;

        let res = client
            .get("/methods")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            serde_json::from_str::<Vec<MFAMethod>>(&res.into_string().await.unwrap()).unwrap(),
            vec![MFAMethod::Password]
        );
    }

    #[async_std::test]
    async fn success_has_recovery_and_totp() {
        use rocket::http::Header;

        let (rauth, session, mut account) =
            for_test_authenticated("get_mfa_methods::success_has_recovery_and_totp").await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "some".to_string(),
        };
        account.mfa.generate_recovery_codes();
        account.save(&rauth).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::mfa::get_mfa_methods::get_mfa_methods],
        )
        .await;

        let res = client
            .get("/methods")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            serde_json::from_str::<Vec<MFAMethod>>(&res.into_string().await.unwrap()).unwrap(),
            vec![MFAMethod::Totp, MFAMethod::Recovery]
        );
    }
}
