//! Confirm an account deletion.
//! PUT /account/delete
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Account Deletion Token
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataAccountDeletion {
    /// Deletion token
    pub token: String,
}

/// # Confirm Account Deletion
///
/// Schedule an account for deletion by confirming the received token.
#[openapi(tag = "Account")]
#[put("/delete", data = "<data>")]
pub async fn confirm_deletion(
    rauth: &State<RAuth>,
    data: Json<DataAccountDeletion>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Find the relevant account
    let mut account = rauth
        .database
        .find_account_with_deletion_token(&data.token)
        .await?;

    // Schedule the account for deletion
    account
        .schedule_deletion(rauth)
        .await
        .map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use chrono::Duration;
    use iso8601_timestamp::Timestamp;

    use crate::test::*;

    #[async_std::test]
    async fn success() {
        let (rauth, _, mut account) = for_test_authenticated("confirm_deletion::success").await;

        account.deletion = Some(DeletionInfo::WaitingForVerification {
            token: "token".into(),
            expiry: Timestamp::from_unix_timestamp_ms(
                chrono::Utc::now()
                    .checked_add_signed(Duration::seconds(100))
                    .expect("failed to checked_add_signed")
                    .timestamp_millis(),
            ),
        });

        account.save(&rauth).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::account::confirm_deletion::confirm_deletion,],
        )
        .await;

        let res = client
            .put("/delete")
            .header(ContentType::JSON)
            .body(
                json!({
                    "token": "token"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
