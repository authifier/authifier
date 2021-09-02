use wither::bson::doc;
use wither::prelude::*;

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(
    collection_name = "sessions",
    index(keys = r#"doc!{"token": 1}"#, options = r#"doc!{"unique": true}"#)
)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub user_id: String,
    pub token: String,
    pub name: String,
}
