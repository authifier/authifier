use rocket::{routes, Route};

pub mod change_email;
pub mod change_password;
pub mod create_account;
pub mod fetch_account;
pub mod password_reset;
pub mod resend_verification;
pub mod send_password_reset;
pub mod verify_email;

pub fn routes() -> Vec<Route> {
    routes![
        create_account::create_account,
        resend_verification::resend_verification,
        fetch_account::fetch_account,
        change_password::change_password,
        change_email::change_email,
        verify_email::verify_email,
        password_reset::password_reset,
        send_password_reset::send_password_reset
    ]
}
