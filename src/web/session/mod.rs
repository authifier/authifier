use rocket::{routes, Route};

mod login;
mod logout;

mod fetch_all;
mod revoke;
mod revoke_all;

pub fn routes() -> Vec<Route> {
    routes![
        login::login,
        logout::logout,
        fetch_all::fetch_all,
        revoke::revoke,
        revoke_all::revoke_all
    ]
}
