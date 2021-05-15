use rocket::Route;

pub mod accounts;
pub mod security;
pub mod sessions;

pub fn routes() -> Vec<Route> {
    routes![
        accounts::change_email::change_email,
        accounts::change_password::change_password,
        accounts::create_account::create_account,
        accounts::fetch_account::fetch_account,
        accounts::verify_account::verify_account,
        security::resend_verification::resend_verification,
        security::send_password_reset::send_password_reset,
        security::reset_password_form::reset_password_form,
        security::reset_password::reset_password,
        sessions::check_session::check_session,
        sessions::create_session::create_session,
        sessions::delete_session::delete_session,
        sessions::fetch_sessions::fetch_sessions,
        sessions::rename_session::rename_session,
        sessions::logout::logout
    ]
}
