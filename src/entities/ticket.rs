use wither::bson::doc;
use wither::prelude::*;

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(
    collection_name = "mfa_tickets",
    index(keys = r#"doc!{"token": 1}"#, options = r#"doc!{"unique": true}"#)
)]
pub struct MFATicket {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub account: String,
    pub token: String,
    pub method: String,
}

impl MFATicket {
    pub fn is_expired() -> bool {
        // decode time from ULID
        // add certain amount of time
        // check if expired

        unimplemented!()
    }

    pub fn claim(self, _token: &str) -> () {
        // check if token is correct
        // remove from db
        // check if expired
        // return new session

        unimplemented!()
    }
}
