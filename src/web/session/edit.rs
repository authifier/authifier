/// Edit a session
/// PATCH /session/:id
use rocket::serde::json::Json;
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{Error, Result};

/// # Edit Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataEditSession {
    /// Session friendly name
    pub friendly_name: String,
}

/// # Edit Session
///
/// Edit current session information.
#[openapi(tag = "Session")]
#[patch("/<id>", data = "<data>")]
pub async fn edit(
    auth: &State<Auth>,
    session: Session,
    id: String,
    data: Json<DataEditSession>,
) -> Result<Json<Session>> {
    let mut session = Session::find_one(
        &auth.db,
        doc! { "_id": id, "user_id": session.user_id },
        None,
    )
    .await
    .map_err(|_| Error::DatabaseError {
        operation: "find_one",
        with: "session",
    })?
    .ok_or(Error::InvalidSession)?;

    session.name = data.into_inner().friendly_name;
    session
        .save(&auth.db, None)
        .await
        .map(|_| Json(session))
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth, session, _) = for_test_authenticated("edit::success").await;
        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::edit::edit]).await;

        let res = client
            .patch(format!("/{}", session.id.as_ref().unwrap()))
            .header(ContentType::JSON)
            .header(Header::new("X-Session-Token", session.token))
            .body(
                json!({
                    "friendly_name": "test name"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        let result = res.into_string().await.unwrap();
        let session = serde_json::from_str::<Session>(&result).unwrap();
        assert_eq!(session.name, "test name");
    }
}
