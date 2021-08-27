use rocket::{routes, Route};

mod create_account;

pub fn routes() -> Vec<Route> {
    routes![create_account::create_account]
}
