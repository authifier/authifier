use rocket::{routes, Route};

pub mod fetch_recovery;
pub mod fetch_status;
pub mod generate_recovery;
pub mod totp_enable;
pub mod totp_generate_secret;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_status::fetch_status,
        // fetch_recovery::fetch_recovery,
        // generate_recovery::generate_recovery,
        // totp_generate_secret::totp_generate_secret,
        // totp_enable::totp_enable,
    ]
}
