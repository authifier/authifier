use crate::{models::Invite, RAuth, Success};

impl Invite {
    /// Save model
    pub async fn save(&self, rauth: &RAuth) -> Success {
        rauth.database.save_invite(self).await
    }
}
