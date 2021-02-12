#![feature(decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket_contrib;

use argon2::Config;

lazy_static! {
    pub static ref ARGON_CONFIG: Config<'static> = Config::default();
}

pub mod auth;
pub mod db;
pub mod email;
pub mod options;
pub mod routes;
pub mod util;
