use rocket::Route;

mod check_session;
mod create_account;
mod create_session;
mod delete_session;
mod fetch_account;
mod fetch_sessions;
mod logout;
mod verify_account;

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
