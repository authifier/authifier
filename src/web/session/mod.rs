use rocket::{routes, Route};

mod login;
mod logout;

pub fn routes() -> Vec<Route> {
    routes![login::login, logout::logout]
}
