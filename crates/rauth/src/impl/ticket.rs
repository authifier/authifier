use crate::{models::MFATicket, RAuth, Success};

impl MFATicket {
    /// Save model
    pub async fn save(&self, rauth: &RAuth) -> Success {
        rauth.database.save_ticket(self).await
    }

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
