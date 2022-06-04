//! Create a new MFA ticket or validate an existing one.
//! PUT /mfa/ticket
use rauth::models::{Account, MFAResponse, MFATicket, UnvalidatedTicket};
use rauth::{Error, RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

#[openapi(tag = "MFA")]
#[put("/ticket", data = "<data>")]
pub async fn create_ticket(
    rauth: &State<RAuth>,
    account: Option<Account>,
    existing_ticket: Option<UnvalidatedTicket>,
    data: Json<MFAResponse>,
) -> Result<Json<MFATicket>> {
    // Find the relevant account
    let mut account = match (account, existing_ticket) {
        (Some(_), Some(_)) => return Err(Error::OperationFailed),
        (Some(account), _) => account,
        (_, Some(ticket)) => {
            rauth.database.delete_ticket(&ticket.id).await?;
            rauth.database.find_account(&ticket.account_id).await?
        }
        _ => return Err(Error::InvalidToken),
    };

    // Validate the MFA response
    account
        .consume_mfa_response(rauth, data.into_inner())
        .await?;

    // Create a new ticket for this account
    MFATicket::new(rauth, account.id, true).await.map(Json)
}

// TODO: write tests
