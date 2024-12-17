//! Fetch recovery codes for an account.
//! POST /mfa/recovery
use authifier::{
    models::{Account, AuthFlow, ValidatedTicket},
    Error, Result,
};
use rocket::serde::json::Json;

/// # Fetch Recovery Codes
///
/// Fetch recovery codes for an account.
#[openapi(tag = "MFA")]
#[post("/recovery")]
pub async fn fetch_recovery(
    account: Account,
    _ticket: ValidatedTicket,
) -> Result<Json<Vec<String>>> {
    let AuthFlow::Password(auth) = &account.auth_flow else {
        return Err(Error::NotAvailable);
    };

    Ok(Json(auth.mfa.recovery_codes.clone()))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (authifier, session, account, _) =
            for_test_authenticated("fetch_recovery::success").await;
        let ticket = MFATicket::new(account.id, true);
        ticket.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![crate::routes::mfa::fetch_recovery::fetch_recovery],
        )
        .await;

        let res = client
            .post("/recovery")
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(
            serde_json::from_str::<Vec<String>>(&res.into_string().await.unwrap())
                .unwrap()
                .is_empty()
        );
    }
}
