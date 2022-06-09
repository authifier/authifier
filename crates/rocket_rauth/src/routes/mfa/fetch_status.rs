//! Fetch MFA status of an account.
//! GET /mfa
use rauth::{
    models::{Account, MultiFactorAuthentication},
    Result,
};
use rocket::serde::json::Json;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Default)]
pub struct MultiFactorStatus {
    email_otp: bool,
    trusted_handover: bool,
    email_mfa: bool,
    totp_mfa: bool,
    security_key_mfa: bool,
    recovery_active: bool,
}

impl From<MultiFactorAuthentication> for MultiFactorStatus {
    fn from(item: MultiFactorAuthentication) -> Self {
        MultiFactorStatus {
            // email_otp: item.enable_email_otp,
            // trusted_handover: item.enable_trusted_handover,
            // email_mfa: item.enable_email_mfa,
            totp_mfa: !item.totp_token.is_disabled(),
            // security_key_mfa: item.security_key_token.is_some(),
            recovery_active: !item.recovery_codes.is_empty(),
            ..Default::default()
        }
    }
}

/// # MFA Status
///
/// Fetch MFA status of an account.
#[openapi(tag = "MFA")]
#[get("/")]
pub async fn fetch_status(account: Account) -> Result<Json<MultiFactorStatus>> {
    Ok(Json(account.mfa.into()))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, _) = for_test_authenticated("fetch_status::success").await;
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::mfa::fetch_status::fetch_status],
        )
        .await;

        let res = client
            .get("/")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(
            serde_json::from_str::<crate::routes::mfa::fetch_status::MultiFactorStatus>(
                &res.into_string().await.unwrap()
            )
            .is_ok()
        );
    }
}
