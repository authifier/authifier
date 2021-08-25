use wither::prelude::*;
use wither::bson::doc;

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(collection_name = "sessions")]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[model(index(index="token", unique="true"))]
    pub token: String,

    pub name: String,
}
