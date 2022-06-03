//! Edit a session
//! PATCH /session/:id
use rauth::models::Session;
use rauth::{Error, RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

use super::fetch_all::SessionInfo;

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
    rauth: &State<RAuth>,
    user: Session,
    id: String,
    data: Json<DataEditSession>,
) -> Result<Json<SessionInfo>> {
    let mut session = rauth.database.find_session(&id).await?;

    // Make sure we own this session
    if user.user_id != session.user_id {
        return Err(Error::InvalidSession);
    }

    // Rename the session
    session.name = data.into_inner().friendly_name;

    // Save session
    rauth.database.save_session(&session).await?;

    Ok(Json(session.into()))
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::{routes::session::fetch_all::SessionInfo, test::*};

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, _) = for_test_authenticated("edit::success").await;
        let client =
            bootstrap_rocket_with_auth(rauth, routes![crate::routes::session::edit::edit]).await;

        let res = client
            .patch(format!("/{}", session.id))
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
        let session = serde_json::from_str::<SessionInfo>(&result).unwrap();
        assert_eq!(session.name, "test name");
    }
}
