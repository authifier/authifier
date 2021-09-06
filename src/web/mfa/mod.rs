use rocket::{routes, Route};

pub mod fetch_status;

pub fn routes() -> Vec<Route> {
    routes![fetch_status::fetch_status,]
}
