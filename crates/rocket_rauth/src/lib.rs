#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
#[cfg(feature = "test")]
#[macro_use]
extern crate serde_json;

pub mod routes;

#[cfg(feature = "test")]
pub mod test;
