/// Multi-factor auth ticket
#[derive(Debug, Serialize, Deserialize)]
pub struct MFATicket {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// Account Id
    pub account_id: String,

    /// Unique Token
    pub token: String,
}
