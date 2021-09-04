/// Change account email.
/// PATCH /account/email
use rocket::serde::json::Json;
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{normalise_email, EmptyResponse, Error, Result};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub email: String,
    pub current_password: String,
}

#[patch("/email", data = "<data>")]
pub async fn change_email(
    auth: &State<Auth>,
    mut account: Account,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Perform validation on given data.
    auth.validate_email(&data.email).await?;
    account.verify_password(&data.current_password)?;

    // Send email verification for new email.
    account.verification = auth
        .generate_email_move_verification(data.email.clone())
        .await;

    // If email verification is disabled, update the email field.
    if let AccountVerification::Verified = &account.verification {
        account.email_normalised = normalise_email(data.email.clone());
        account.email = data.email;
    }

    // Commit to database.
    account
        .save(&auth.db, None)
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "account",
        })?;

    Ok(EmptyResponse)
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (db, auth, session, account) = for_test_authenticated("change_email::success").await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::change_email::change_email],
        )
        .await;

        let res = client
            .patch("/email")
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "email": "validexample@valid.com",
                    "current_password": "password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let account = Account::find_one(&db, doc! { "_id": account.id.unwrap() }, None)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(account.email, "validexample@valid.com");
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success_smtp() {
        use rocket::http::Header;

        let (db, auth, session, account) = for_test_authenticated_with_config(
            "change_email::success_smtp",
            test_smtp_config().await,
        )
        .await;
        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::change_email::change_email],
        )
        .await;

        let res = client
            .patch("/email")
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "email": "validexample@valid.com",
                    "current_password": "password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let account = Account::find_one(&db, doc! { "_id": account.id.unwrap() }, None)
            .await
            .unwrap()
            .unwrap();

        assert_ne!(account.email, "validexample@valid.com");
    }
}
