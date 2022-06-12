//! Generate a new secret for TOTP.
//! POST /mfa/totp
use rauth::models::{Account, MFAResponse};
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Enable TOTP 2FA
///
/// Generate a new secret for TOTP.
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
mod tests {
    use rauth::models::totp::Totp;

    use crate::{routes::session::login::ResponseLogin, test::*};

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, account) = for_test_authenticated("totp_enable::success").await;
        let ticket = MFATicket::new(&rauth, account.id.to_string(), true)
            .await
            .unwrap();
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![
                crate::routes::mfa::totp_generate_secret::totp_generate_secret,
                crate::routes::mfa::totp_enable::totp_enable,
                crate::routes::session::login::login
            ],
        )
        .await;

        let res = client
            .post("/totp")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .header(Header::new("X-MFA-Ticket", ticket.token))
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

        let code = Totp::Enabled { secret }.generate_code().unwrap();

        let res = client
            .put("/totp")
            .header(Header::new("X-Session-Token", session.token))
            .header(ContentType::JSON)
            .body(json!({ "totp_code": code }).to_string())
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "email@revolt.chat",
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        let response =
            serde_json::from_str::<ResponseLogin>(&res.into_string().await.unwrap()).unwrap();

        if let ResponseLogin::MFA { ticket, .. } = response {
            let res = client
                .post("/login")
                .header(ContentType::JSON)
                .body(
                    json!({
                        "mfa_ticket": ticket,
                        "mfa_response": {
                            "totp_code": code
                        }
                    })
                    .to_string(),
                )
                .dispatch()
                .await;

            assert_eq!(res.status(), Status::Ok);
        } else {
            unreachable!("Did not receive MFA challenge!");
        }
    }
}
