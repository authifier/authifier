use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

pub mod edit;
pub mod fetch_all;
pub mod login;
pub mod logout;
pub mod revoke;
pub mod revoke_all;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        login::login,
        logout::logout,
        fetch_all::fetch_all,
        revoke::revoke,
        revoke_all::revoke_all,
        edit::edit
    ]
}
