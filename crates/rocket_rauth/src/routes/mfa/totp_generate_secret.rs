//! Generate a new secret for TOTP.
//! POST /mfa/totp
use rauth::models::{Account, ValidatedTicket};
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

#[derive(Serialize, JsonSchema)]
pub struct ResponseTotpSecret {
    secret: String,
}

#[openapi(tag = "MFA")]
#[post("/totp")]
pub async fn totp_generate_secret(
    rauth: &State<RAuth>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<Json<ResponseTotpSecret>> {
    // Generate a new secret
    let secret = account.mfa.generate_new_totp_secret()?;

    // Save model to database
    account.save(rauth).await?;

    // Send secret to user
    Ok(Json(ResponseTotpSecret { secret }))
}

#[cfg(test)]
#[cfg(feature = "test")]
#[cfg(feature = "TODO")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("totp_generate_secret::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::routes::mfa::totp_generate_secret::totp_generate_secret],
        )
        .await;

        let res = client
            .post("/totp")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        #[allow(dead_code)]
        #[derive(Deserialize)]
        pub struct Response {
            secret: String,
        }

        assert!(
            serde_json::from_str::<ResponseTotpSecret>(&res.into_string().await.unwrap()).is_ok()
        );
    }
}
