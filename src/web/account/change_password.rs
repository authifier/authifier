/// Change account password.
/// PATCH /account/change/password
use rocket::serde::json::Json;
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Result};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub password: String,
    pub current_password: String,
}

#[patch("/change/password", data = "<data>")]
pub async fn change_password(
    auth: &State<Auth>,
    mut account: Account,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Perform validation on given data.
    auth.validate_password(&data.password).await?;
    account.verify_password(&data.current_password)?;

    // Hash and replace password.
    account.password = auth.hash_password(data.password)?;

    // Commit to database.
    account.save_to_db(&auth.db).await.map(|_| EmptyResponse)
}

#[cfg(test)]
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
