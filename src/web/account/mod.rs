use rocket::{routes, Route};

mod create_account;
mod resend_verification;

mod change_password;
mod fetch_account;

pub fn routes() -> Vec<Route> {
    routes![
        create_account::create_account,
        resend_verification::resend_verification,
        fetch_account::fetch_account,
        change_password::change_password
    ]
}
