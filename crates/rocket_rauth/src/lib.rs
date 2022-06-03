#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;

// Rust compiler seems to think
// this isn't used even though
// it is used.
#[allow(unused_imports)]
#[cfg(feature = "test")]
#[macro_use]
extern crate serde_json;

pub mod routes;

#[cfg(feature = "test")]
pub mod test;
