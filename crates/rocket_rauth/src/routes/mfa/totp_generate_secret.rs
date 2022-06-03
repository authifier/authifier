//! Generate a new secret for TOTP.
//! POST /mfa/totp
use rauth::models::Account;
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

#[derive(Serialize)]
pub struct Response {
    secret: String,
}

#[post("/totp")]
pub async fn totp_generate_secret(
    rauth: &State<RAuth>,
    mut account: Account,
) -> Result<Json<Response>> {
    /*let secret = auth.mfa_generate_totp_secret(&mut account).await?;
    Ok(Json(Response { secret }))*/
    todo!()
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("totp_generate_secret::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::mfa::totp_generate_secret::totp_generate_secret],
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

        assert!(serde_json::from_str::<Response>(&res.into_string().await.unwrap()).is_ok());
    }
}
