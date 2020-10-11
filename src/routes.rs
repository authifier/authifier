use rocket::{ Rocket, State };
use rocket_contrib::json::Json;
use super::auth::{ Auth, Create };

#[post("/create", data = "<data>")]
fn create(auth: State<Auth>, data: Json<Create>) {
    match auth.inner().create_account(data.into_inner()) {
        Ok(_) => (),
        Err(error) => {
            
        }
    }
}

pub fn mount(rocket: Rocket, path: &str, auth: Auth) -> Rocket {
    rocket.manage(auth)
        .mount(path, routes![ create ])
}
