use rocket::{routes, Route};

mod change_email;
mod change_password;
mod create_account;
mod fetch_account;
mod resend_verification;

pub fn routes() -> Vec<Route> {
    routes![
        create_account::create_account,
        resend_verification::resend_verification,
        fetch_account::fetch_account,
        change_password::change_password,
        change_email::change_email
    ]
}
