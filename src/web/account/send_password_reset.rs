/// Send a password reset email
/// POST /account/reset_password
use mongodb::bson::doc;
use mongodb::options::{Collation, FindOneOptions};
use rocket::serde::json::Json;
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Error, Result};

/// # Reset Information
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataSendPasswordReset {
    /// Email associated with the account
    pub email: String,
    /// Captcha verification code
    pub captcha: Option<String>,
}

/// # Send Password Reset
///
/// Send an email to reset account password.
#[openapi(tag = "Account")]
#[post("/reset_password", data = "<data>")]
pub async fn send_password_reset(
    auth: &State<Auth>,
    data: Json<DataSendPasswordReset>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();

    // Perform validation on given data.
    auth.check_captcha(data.captcha).await?;
    auth.validate_email(&data.email).await?;

    // From this point on, do not report failure to the
    // remote client, as this will open us up to user enumeration.

    // Try to find the relevant account.
    if let Some(mut account) = Account::find_one(
        &auth.db,
        doc! { "email": data.email },
        FindOneOptions::builder()
            .collation(Collation::builder().locale("en").strength(2).build())
            .build(),
    )
    .await
    .map_err(|_| Error::DatabaseError {
        operation: "find_one",
        with: "account",
    })? {
        // Generate password reset email.
        account.password_reset = auth
            .generate_email_password_reset(account.email.clone())
            .await;

        // Commit to database.
        account.save_to_db(&auth.db).await?;
    }

    // Never fail this route, (except for db error)
    // You may open the application to email enumeration otherwise.
    Ok(EmptyResponse)
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use chrono::Utc;
        use mongodb::bson::DateTime;

        let (db, auth) =
            for_test_with_config("send_password_reset::success", test_smtp_config().await).await;

        let mut account = auth
            .create_account("password_reset@smtp.test".into(), "password".into(), false)
            .await
            .unwrap();

        account.verification = AccountVerification::Pending {
            token: "".into(),
            expiry: DateTime(Utc::now()),
        };

        account.save(&db, None).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            auth,
            routes![
                crate::web::account::password_reset::password_reset,
                crate::web::account::send_password_reset::send_password_reset,
                crate::web::session::login::login
            ],
        )
        .await;

        let res = client
            .post("/reset_password")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "password_reset@smtp.test",
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let mail = assert_email_sendria("password_reset@smtp.test".into()).await;
        let res = client
            .patch("/reset_password")
            .header(ContentType::JSON)
            .body(
                json!({
                    "token": mail.code.expect("`code`"),
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);

        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(
                json!({
                    "email": "password_reset@smtp.test",
                    "password": "valid password"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(serde_json::from_str::<Session>(&res.into_string().await.unwrap()).is_ok());
    }
}
