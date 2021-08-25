use wither::prelude::*;
use wither::bson::doc;

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(collection_name = "mfa_tickets")]
pub struct MFATicket {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub account: String,
    #[model(index(index="token", unique="true"))]
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

    pub fn claim(self, token: &str) -> () {
        // check if token is correct
        // remove from db
        // check if expired
        // return new session

        unimplemented!()
    }
}
