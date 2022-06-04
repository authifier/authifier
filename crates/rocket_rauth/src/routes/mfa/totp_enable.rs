//! Generate a new secret for TOTP.
//! POST /mfa/totp
use rauth::models::{Account, MFAResponse};
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "MFA")]
#[put("/totp", data = "<data>")]
pub async fn totp_enable(
    rauth: &State<RAuth>,
    mut account: Account,
    data: Json<MFAResponse>,
) -> Result<EmptyResponse> {
    // Enable TOTP 2FA
    account.mfa.enable_totp(data.into_inner())?;

    // Save model to database
    account.save(rauth).await.map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
#[cfg(feature = "TODO")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("totp_enable::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::routes::mfa::totp_generate_secret::totp_generate_secret,
                crate::routes::mfa::totp_enable::totp_enable
            ],
        )
        .await;

        let res = client
            .post("/totp")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        #[allow(dead_code)]
        #[derive(Deserialize)]
        pub struct Response {
            secret: String,
        }

        let Response { secret } =
            serde_json::from_str::<Response>(&res.into_string().await.unwrap()).unwrap();

        let secret_u8 = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
            .expect("valid `secret`");

        let code = Auth::mfa_generate_totp_code(&secret_u8);
        let res = client
            .put("/totp")
            .header(Header::new("X-Session-Token", session.token))
            .header(ContentType::JSON)
            .body(json!({ "code": code, "password": "password_insecure" }).to_string())
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
