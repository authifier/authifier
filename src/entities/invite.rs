use mongodb::Database;
use wither::bson::doc;
use wither::prelude::*;

use crate::util::{Error, Result};

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(collection_name = "invites")]
pub struct Invite {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
}

impl Invite {
    pub async fn claim(mut self, db: &Database, id: String) -> Result<()> {
        self.used = Some(true);
        self.claimed_by = Some(id);
        self.save(&db, None)
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "save",
                with: "invite",
            })
    }
}
