use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountVerification {
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSession {
    pub token: String,
    pub friendly_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub password: String,
}
