use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountVerification {
    pub verified: bool,
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
    pub friendly_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub password: String,
    pub verification: AccountVerification,
    pub sessions: Vec<AccountSession>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountShort {
    pub id: String,
    pub email: String,
    pub verified: bool
}
