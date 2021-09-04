/// Verify an account
/// POST /verify/<code>
use chrono::Utc;
use mongodb::bson::Bson;
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Error, Result};

#[post("/verify/<code>")]
pub async fn verify_email(auth: &State<Auth>, code: String) -> Result<EmptyResponse> {
    let mut account = Account::find_one(
        &auth.db,
        doc! {
            "verification.token": &code,
            "verification.expiry": {
                "$gte": Bson::DateTime(Utc::now())
            }
        },
        None,
    )
    .await
    .map_err(|_| Error::DatabaseError {
        operation: "find_one",
        with: "account",
    })?
    .ok_or_else(|| Error::InvalidToken)?;

    auth.verify_account(&mut account)
        .await
        .map(|_| EmptyResponse)
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use chrono::{Duration, Utc};
        use mongodb::bson::DateTime;

        let (db, auth, _, mut account) = for_test_authenticated("verify_email::success").await;

        account.verification = AccountVerification::Pending {
            token: "token".into(),
            expiry: DateTime(
                Utc::now()
                    .checked_add_signed(Duration::seconds(60))
                    .expect("failed to checked_add_signed"),
            ),
        };

        account.save(&db, None).await.unwrap();

        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::verify_email::verify_email],
        )
        .await;

        let res = client.post("/verify/token").dispatch().await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_token() {
        let (_, auth) = for_test("verify_email::fail_invalid_token").await;

        let client = bootstrap_rocket_with_auth(
            auth,
            routes![crate::web::account::verify_email::verify_email],
        )
        .await;

        let res = client.post("/verify/token").dispatch().await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(
            res.into_string().await,
            Some("{\"type\":\"InvalidToken\"}".into())
        );
    }
}
