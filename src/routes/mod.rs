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
        
        sessions::check_session::check_session,
        sessions::create_session::create_session,
        sessions::delete_session::delete_session,
        sessions::fetch_sessions::fetch_sessions,
        sessions::logout::logout
    ]
}
