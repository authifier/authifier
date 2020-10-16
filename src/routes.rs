use rocket::{ Rocket, State };
use rocket_contrib::json::{ Json, JsonValue };
use super::auth::{ Auth, Create, Verify };

#[post("/create", data = "<data>")]
async fn create(auth: State<'_, Auth>, data: Json<Create>) -> super::util::Result<JsonValue> {
    /*match auth.inner().create_account(data.into_inner()) {
        Ok(_) => {
            json!({
                "bruh": true
            })
        },
        Err(error) => {
            json!({
                "error": error
            })
        }
    }*/

    auth.inner().create_account(data.into_inner()).await?;

    Ok(json!({ "nice": true }))
}

#[get("/verify/<code>")]
fn verify(auth: State<Auth>, code: String) -> super::util::Result<JsonValue> {
    auth.inner().verify_account(Verify { code })?;

    Ok(json!({ "nice": true }))
}

pub fn mount(rocket: Rocket, path: &str, auth: Auth) -> Rocket {
    rocket.manage(auth)
        .mount(path, routes![ create ])
}
