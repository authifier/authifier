use rocket::Route;

pub mod check_session;
pub mod create_account;
pub mod create_session;
pub mod delete_session;
pub mod fetch_account;
pub mod fetch_sessions;
pub mod logout;
pub mod verify_account;

pub fn routes() -> Vec<Route> {
    routes![
        check_session::check_session,
        create_account::create_account,
        create_session::create_session,
        delete_session::delete_session,
        fetch_account::fetch_account,
        fetch_sessions::fetch_sessions,
        logout::logout,
        verify_account::verify_account
    ]
}
