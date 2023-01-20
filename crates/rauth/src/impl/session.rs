use crate::{models::Session, RAuth, RAuthEvent, Success};

impl Session {
    /// Save model
    pub async fn save(&self, rauth: &RAuth) -> Success {
        rauth.database.save_session(self).await
    }

    /// Delete session
    pub async fn delete(self, rauth: &RAuth) -> Success {
        // Delete from database
        rauth.database.delete_session(&self.id).await?;

        // Create and push event
        rauth
            .publish_event(RAuthEvent::DeleteSession {
                user_id: self.user_id,
                session_id: self.id,
            })
            .await;

        Ok(())
    }
}
