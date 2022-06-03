#[macro_use]
extern crate serde;
#[macro_use]
extern crate schemars;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate nanoid;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

#[cfg(feature = "database-mongodb")]
#[macro_use]
extern crate bson;

mod result;
pub use result::*;

pub mod config;
pub mod database;
pub mod derive;
pub mod r#impl;
pub mod models;
pub mod util;

pub use config::Config;
pub use database::{Database, Migration};

/// rAuth state
#[derive(Default, Clone)]
pub struct RAuth {
    pub config: Config,
    pub database: Database,
}
