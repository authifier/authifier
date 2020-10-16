#![feature(decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket_contrib;

pub mod routes;
pub mod auth;
pub mod util;

/*pub struct Session {
    user_id: String,
    session_token: String
}*/

/*pub trait Auth {
    fn create_account(email: String, username: String, password: String) -> Result<String>;
    fn verify_account(code: &str) -> Result<String>;
    fn fetch_verification(email: String) -> Result<String>;
    fn login(email: String, password: String) -> Result<Session>;
    fn verify_session(session: &Session) -> Result<bool>;
}*/
