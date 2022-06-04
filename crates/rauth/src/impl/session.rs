use crate::{models::Session, RAuth, Success};

impl Session {
    /// Save model
    pub async fn save(&self, rauth: &RAuth) -> Success {
        rauth.database.save_session(self).await
    }
}
