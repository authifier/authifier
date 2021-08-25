use wither::prelude::*;
use wither::bson::doc;

#[derive(Debug, Serialize, Deserialize)]
#[model(collection_name = "invites")]
pub struct Invite {
    #[serde(rename = "_id")]
    pub id: String,
    pub used: Option<bool>,
    pub claimed_by: Option<String>,
}