/// Fetch your account
/// GET /account
use rocket::serde::json::Json;

use crate::entities::*;
use crate::util::Result;

/// # Fetch Account
/// 
/// Fetch account information from the current session.
#[openapi(tag = "Account")]
#[get("/")]
pub async fn fetch_account(account: Account) -> Result<Json<AccountInfo>> {
    Ok(Json(account.into()))
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("fetch_account::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::fetch_account::fetch_account],
        )
        .await;

        let res = client
            .get("/")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<AccountInfo>(&res.into_string().await.unwrap()).is_ok());
    }
}
