/// Fetch MFA status of an account.
/// GET /mfa
use rocket::serde::json::Json;

use crate::entities::*;
use crate::util::Result;

#[get("/")]
pub async fn fetch_status(account: Account) -> Result<Json<MultiFactorStatus>> {
    Ok(Json(account.mfa.into()))
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("fetch_status::success").await;
        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::mfa::fetch_status::fetch_status])
                .await;

        let res = client
            .get("/")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(
            serde_json::from_str::<MultiFactorStatus>(&res.into_string().await.unwrap()).is_ok()
        );
    }
}
