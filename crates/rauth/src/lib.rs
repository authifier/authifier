#[macro_use]
extern crate serde;
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

#[cfg(feature = "schemas")]
#[macro_use]
extern crate schemars;
#[cfg(feature = "database-mongodb")]
#[macro_use]
extern crate bson;

mod result;
pub use result::*;

pub mod config;
pub mod database;
pub mod derive;
pub mod events;
pub mod r#impl;
pub mod models;
pub mod util;

pub use config::Config;
pub use database::{Database, Migration};
pub use events::AuthifierEvent;

use async_std::channel::Sender;

/// Authifier state
#[derive(Default, Clone)]
pub struct Authifier {
    pub config: Config,
    pub database: Database,
    pub event_channel: Option<Sender<AuthifierEvent>>,
}

impl Authifier {
    pub async fn publish_event(&self, event: AuthifierEvent) {
        if let Some(sender) = &self.event_channel {
            if let Err(err) = sender.send(event).await {
                error!("Failed to publish an Authifier event: {:?}", err);
            }
        }
    }
}
