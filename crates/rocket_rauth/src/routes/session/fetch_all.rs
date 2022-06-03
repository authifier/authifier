//! Fetch all sessions
//! GET /session/all
use rauth::models::Session;
use rauth::{RAuth, Result};
use rocket::serde::json::Json;
use rocket::State;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SessionInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl From<Session> for SessionInfo {
    fn from(item: Session) -> Self {
        SessionInfo {
            id: item.id,
            name: item.name,
        }
    }
}

/// # Fetch Sessions
///
/// Fetch all sessions associated with this account.
#[openapi(tag = "Session")]
#[get("/all")]
pub async fn fetch_all(rauth: &State<RAuth>, session: Session) -> Result<Json<Vec<SessionInfo>>> {
    rauth
        .database
        .find_sessions(&session.user_id)
        .await
        .map(|ok| ok.into_iter().map(|session| session.into()).collect())
        .map(Json)
}

#[cfg(test)]
#[cfg(feature = "test")]
mod tests {
    use crate::test::*;

    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (rauth, session, account) = for_test_authenticated("fetch_all::success").await;

        for i in 1..=3 {
            account
                .create_session(&rauth, format!("session{}", i))
                .await
                .unwrap();
        }

        let client = bootstrap_rocket_with_auth(
            rauth,
            routes![crate::routes::session::fetch_all::fetch_all],
        )
        .await;

        let res = client
            .get("/all")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        let result = res.into_string().await.unwrap();
        let sessions: Vec<crate::routes::session::fetch_all::SessionInfo> =
            serde_json::from_str(&result).unwrap();
        assert_eq!(sessions.len(), 4);
    }
}
