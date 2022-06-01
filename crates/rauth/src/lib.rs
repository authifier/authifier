#[macro_use]
extern crate serde;
#[macro_use]
extern crate schemars;
#[macro_use]
extern crate lazy_static;

mod result;
pub use result::*;

pub mod config;
pub mod r#impl;
pub mod models;
