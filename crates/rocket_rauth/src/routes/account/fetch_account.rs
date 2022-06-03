//! Fetch your account
//! GET /account
use rauth::{models::Account, Result};
use rocket::serde::json::Json;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
}

impl From<Account> for AccountInfo {
    fn from(item: Account) -> Self {
        AccountInfo {
            id: item.id,
            email: item.email,
        }
    }
}

/// # Fetch Account
///
/// Fetch account information from the current session.
#[openapi(tag = "Account")]
#[get("/")]
pub async fn fetch_account(account: Account) -> Result<Json<AccountInfo>> {
    Ok(Json(account.into()))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, _) = for_test_authenticated("fetch_account::success").await;
        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::account::fetch_account::fetch_account],
        )
        .await;

        let res = client
            .get("/")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(
            serde_json::from_str::<crate::routes::account::fetch_account::AccountInfo>(
                &res.into_string().await.unwrap()
            )
            .is_ok()
        );
    }
}
