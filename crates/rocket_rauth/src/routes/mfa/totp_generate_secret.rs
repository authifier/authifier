//! Generate a new secret for TOTP.
//! POST /mfa/totp
use authifier::models::{Account, ValidatedTicket};
use authifier::{Authifier, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Totp Secret
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResponseTotpSecret {
    secret: String,
}

/// # Generate TOTP Secret
///
/// Generate a new secret for TOTP.
#[openapi(tag = "MFA")]
#[post("/totp")]
pub async fn totp_generate_secret(
    authifier: &State<Authifier>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<Json<ResponseTotpSecret>> {
    // Generate a new secret
    let secret = account.mfa.generate_new_totp_secret()?;

    // Save model to database
    account.save(authifier).await?;

    // Send secret to user
    Ok(Json(ResponseTotpSecret { secret }))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::routes::mfa::totp_generate_secret::ResponseTotpSecret;
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (authifier, session, account, _) =
            for_test_authenticated("totp_generate_secret::success").await;
        let ticket = MFATicket::new(account.id.to_string(), true);
        ticket.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier.clone(),
            routes![crate::routes::mfa::totp_generate_secret::totp_generate_secret],
        )
        .await;

        let res = client
            .post("/totp")
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        let ResponseTotpSecret { secret } =
            serde_json::from_str::<ResponseTotpSecret>(&res.into_string().await.unwrap()).unwrap();

        let account = authifier.database.find_account(&account.id).await.unwrap();
        assert_eq!(account.mfa.totp_token, Totp::Pending { secret });
    }
}
