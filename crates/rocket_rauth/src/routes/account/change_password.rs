use rauth::Result;
/// Change account password.
/// PATCH /account/change/password
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Change Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataChangePassword {
    /// New password
    pub password: String,
    /// Current password
    pub current_password: String,
}

/// # Change Password
///
/// Change the current account password.
#[openapi(tag = "Account")]
#[patch("/change/password", data = "<data>")]
pub async fn change_password(
    /*auth: &State<Auth>,
    mut account: Account,*/
    data: Json<DataChangePassword>,
) -> Result<EmptyResponse> {
    /*let data = data.into_inner();

    // Perform validation on given data.
    auth.validate_password(&data.password).await?;
    account.verify_password(&data.current_password)?;

    // Hash and replace password.
    account.password = auth.hash_password(data.password)?;

    // Commit to database.
    account.save_to_db(&auth.db).await.map(|_| EmptyResponse)*/
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

        let (_, auth, session, _) = for_test_authenticated("change_password::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::change_password::change_password],
        )
        .await;

        let res = client
            .patch("/change/password")
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "password": "new password",
                    "current_password": "password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let res = client
            .patch("/change/password")
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token))
            .body(
                json!({
                    "password": "sussy password",
                    "current_password": "new password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }
}
