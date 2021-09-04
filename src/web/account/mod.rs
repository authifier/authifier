use rocket::{routes, Route};

mod create_account;
mod resend_verification;

pub fn routes() -> Vec<Route> {
    routes![
        create_account::create_account,
        resend_verification::resend_verification
    ]
}
