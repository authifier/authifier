//! Create a new MFA ticket or validate an existing one.
//! PUT /mfa/ticket
use authifier::models::{Account, MFAResponse, MFATicket, UnvalidatedTicket};
use authifier::{Authifier, Error, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Create MFA ticket
///
/// Create a new MFA ticket or validate an existing one.
#[openapi(tag = "MFA")]
#[put("/ticket", data = "<data>")]
pub async fn create_ticket(
    authifier: &State<Authifier>,
    account: Option<Account>,
    existing_ticket: Option<UnvalidatedTicket>,
    data: Json<MFAResponse>,
) -> Result<Json<MFATicket>> {
    // Find the relevant account
    let mut account = match (account, existing_ticket) {
        (Some(_), Some(_)) => return Err(Error::OperationFailed),
        (Some(account), _) => account,
        (_, Some(ticket)) => {
            authifier.database.delete_ticket(&ticket.id).await?;
            authifier.database.find_account(&ticket.account_id).await?
        }
        _ => return Err(Error::InvalidToken),
    };

    // Validate the MFA response
    account
        .consume_mfa_response(authifier, data.into_inner(), None)
        .await?;

    // Create a new ticket for this account
    let ticket = MFATicket::new(account.id, true);
    ticket.save(authifier).await?;
    Ok(Json(ticket))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (authifier, session, _, _) = for_test_authenticated("create_ticket::success").await;
        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![crate::routes::mfa::create_ticket::create_ticket,],
        )
        .await;

        let res = client
            .put("/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        serde_json::from_str::<MFATicket>(&res.into_string().await.unwrap()).unwrap();
    }

    #[async_std::test]
    async fn success_totp() {
        use rocket::http::Header;

        let (authifier, session, mut account, _) =
            for_test_authenticated("create_ticket::success_totp").await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "secret".to_string(),
        };
        account.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![crate::routes::mfa::create_ticket::create_ticket,],
        )
        .await;

        let res = client
            .put("/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "totp_code": Totp::Enabled {
                        secret: "secret".to_string(),
                    }.generate_code().unwrap()
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        serde_json::from_str::<MFATicket>(&res.into_string().await.unwrap()).unwrap();
    }

    #[async_std::test]
    async fn failure_totp() {
        use rocket::http::Header;

        let (authifier, session, mut account, _) =
            for_test_authenticated("create_ticket::failure_totp").await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "secret".to_string(),
        };
        account.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![crate::routes::mfa::create_ticket::create_ticket,],
        )
        .await;

        let res = client
            .put("/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "totp_code": "000000"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidToken\"}".into())
        );
    }

    #[async_std::test]
    async fn failure_no_totp() {
        use rocket::http::Header;

        let (authifier, session, mut account, _) =
            for_test_authenticated("create_ticket::failure_no_totp").await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "secret".to_string(),
        };
        account.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
            routes![crate::routes::mfa::create_ticket::create_ticket,],
        )
        .await;

        let res = client
            .put("/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "password": "this is the wrong mfa method"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"DisallowedMFAMethod\"}".into())
        );
    }
}
