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

    /// ???
    pub method: String,
}

impl MFATicket {
    pub fn is_expired() -> bool {
        // decode time from ULID
        // add certain amount of time
        // check if expired

        unimplemented!()
    }

    pub fn claim(self, _token: &str) {
        // check if token is correct
        // remove from db
        // check if expired
        // return new session

        unimplemented!()
    }
}
