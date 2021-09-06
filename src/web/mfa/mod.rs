use rocket::{routes, Route};

pub mod fetch_status;
pub mod fetch_recovery;
pub mod generate_recovery;

pub fn routes() -> Vec<Route> {
    routes![fetch_status::fetch_status,fetch_recovery::fetch_recovery,generate_recovery::generate_recovery]
}
