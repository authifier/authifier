use rauth::Result;
/// Fetch recovery codes for an account.
/// POST /mfa/recovery
use rocket::serde::json::Json;

#[derive(Serialize, Deserialize)]
pub struct Data {
    password: String,
}

#[post("/recovery", data = "<data>")]
pub async fn fetch_recovery(/*account: Account,*/ data: Json<Data>,) -> Result<Json<Vec<String>>> {
    /*account.verify_password(&data.password)?;
    Ok(Json(account.mfa.recovery_codes))*/
    todo!()
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("fetch_recovery::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::mfa::fetch_recovery::fetch_recovery],
        )
        .await;

        let res = client
            .post("/recovery")
            .header(Header::new("X-Session-Token", session.token))
            .header(ContentType::JSON)
            .body(
                json!({
                    "password": "password",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Vec<String>>(&res.into_string().await.unwrap()).is_ok());
    }
}
