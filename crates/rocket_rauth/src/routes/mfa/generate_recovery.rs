//! Re-generate recovery codes for an account.
//! PATCH /mfa/recovery
use rauth::models::Account;
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

#[derive(Serialize, Deserialize)]
pub struct Data {
    password: String,
}

#[patch("/recovery", data = "<data>")]
pub async fn generate_recovery(
    rauth: &State<RAuth>,
    mut account: Account,
    data: Json<Data>,
) -> Result<Json<Vec<String>>> {
    /*account.verify_password(&data.password)?;
    auth.mfa_regenerate_recovery(&mut account).await?;
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

        let (_, auth, session, _) = for_test_authenticated("generate_recovery::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::web::mfa::generate_recovery::generate_recovery,
                crate::web::mfa::fetch_recovery::fetch_recovery
            ],
        )
        .await;

        let res = client
            .patch("/recovery")
            .header(Header::new("X-Session-Token", session.token.clone()))
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
        assert_eq!(
            serde_json::from_str::<Vec<String>>(&res.into_string().await.unwrap())
                .unwrap()
                .len(),
            10
        );
    }
}
