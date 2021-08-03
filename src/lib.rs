#![feature(decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use argon2::Config;

lazy_static! {
    pub static ref ARGON_CONFIG: Config<'static> = Config::default();
    pub static ref COMPROMISED_PASSWORDS: Vec<&'static str> = include_str!("../pwned100k.txt").split('\n').skip(4).collect();
}

pub mod auth;
pub mod db;
pub mod email;
pub mod options;
pub mod routes;
pub mod util;
