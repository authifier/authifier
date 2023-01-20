use crate::{models::Invite, Authifier, Success};

impl Invite {
    /// Save model
    pub async fn save(&self, authifier: &Authifier) -> Success {
        authifier.database.save_invite(self).await
    }
}
