use rauth::Result;
/// Fetch all sessions
/// GET /session/all
use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Sessions
///
/// Fetch all sessions associated with this account.
#[openapi(tag = "Session")]
#[get("/all")]
pub async fn fetch_all(/*auth: &State<Auth>, session: Session*/
) -> Result<Json<Vec</*SessionInfo*/ ()>>> {
    /*let mut cursor = Session::find(&auth.db, doc! { "user_id": session.user_id }, None)
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "sessions",
        })?;

    let mut list = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(session) = result {
            list.push(session.into());
        }
    }

    Ok(Json(list))*/
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

        let (_, auth, session, account) = for_test_authenticated("fetch_all::success").await;

        for i in 1..=3 {
            auth.create_session(&account, format!("session{}", i))
                .await
                .unwrap();
        }

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::fetch_all::fetch_all])
                .await;

        let res = client
            .get("/all")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);

        let result = res.into_string().await.unwrap();
        let sessions: Vec<SessionInfo> = serde_json::from_str(&result).unwrap();
        assert_eq!(sessions.len(), 4);
    }
}
