use revolt_okapi::openapi3::OpenApi;
use rocket::Route;

pub mod authorize;
pub mod callback;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![authorize::authorize, callback::callback]
}
