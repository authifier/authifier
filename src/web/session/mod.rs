use rocket::{routes, Route};

mod edit;
mod fetch_all;
mod login;
mod logout;
mod revoke;
mod revoke_all;

pub fn routes() -> Vec<Route> {
    routes![
        login::login,
        logout::logout,
        fetch_all::fetch_all,
        revoke::revoke,
        revoke_all::revoke_all,
        edit::edit
    ]
}
