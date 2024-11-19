//! Confirm an account deletion.
//! PUT /account/delete
use authifier::{Authifier, Result};
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
    authifier: &State<Authifier>,
    data: Json<DataAccountDeletion>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Find the relevant account
    let mut account = authifier
        .database
        .find_account_with_deletion_token(&data.token)
        .await?;

    // Schedule the account for deletion
    account
        .schedule_deletion(authifier)
        .await
        .map(|_| EmptyResponse)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use iso8601_timestamp::{Duration, Timestamp};

    use crate::test::*;

    #[tokio::test]
    async fn success() {
        let (authifier, _, mut account, _) =
            for_test_authenticated("confirm_deletion::success").await;

        account.deletion = Some(DeletionInfo::WaitingForVerification {
            token: "token".into(),
            expiry: Timestamp::now_utc()
                .checked_add(Duration::seconds(100))
                .expect("failed to checked_add"),
        });

        account.save(&authifier).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            authifier,
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
