#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate rocket_okapi;

pub mod config;
pub mod entities;
pub mod logic;
pub mod util;
pub mod web;

#[cfg(test)]
pub mod test;
