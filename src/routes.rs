use rocket::{ Rocket, State };
use rocket_contrib::json::{ Json, JsonValue };
use super::auth::{ Auth, Create, Verify, Login, Session };

#[post("/create", data = "<data>")]
async fn create(auth: State<'_, Auth>, data: Json<Create>) -> super::util::Result<JsonValue> {
    let user_id = auth.inner().create_account(data.into_inner()).await?;

    Ok(json!({
        "user_id": user_id
    }))
}

#[get("/verify/<code>")]
async fn verify(auth: State<'_, Auth>, code: String) -> super::util::Result<()> {
    auth.inner().verify_account(Verify { code }).await?;

    Ok(())
}

#[post("/login", data = "<data>")]
async fn login(auth: State<'_, Auth>, data: Json<Login>) -> super::util::Result<JsonValue> {
    let session = auth.inner().login(data.into_inner()).await?;

    Ok(json!(session))
}

#[get("/check")]
async fn check(_session: Session) -> super::util::Result<()> {
    Ok(())
}

pub fn mount(rocket: Rocket, path: &str, auth: Auth) -> Rocket {
    rocket.manage(auth)
        .mount(path, routes![ create, verify, login, check ])
}
