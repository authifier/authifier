use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AccountVerification {
    Verified,
    Pending {
        token: String,
        expiry: DateTime,
        rate_limit: DateTime,
    },
    Moving {
        new_email: String,
        token: String,
        expiry: DateTime,
        rate_limit: DateTime,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSession {
    pub id: String,
    pub token: String,
    pub friendly_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSessionInfo {
    pub id: String,
    pub friendly_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub email_normalised: String,
    pub password: String,
    pub verification: AccountVerification,
    pub sessions: Vec<AccountSession>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountShort {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub verification: AccountVerification,
}
