use crate::{models::Session, Authifier, AuthifierEvent, Success};

impl Session {
    /// Save model
    pub async fn save(&self, authifier: &Authifier) -> Success {
        authifier.database.save_session(self).await
    }

    /// Delete session
    pub async fn delete(self, authifier: &Authifier) -> Success {
        // Delete from database
        authifier.database.delete_session(&self.id).await?;

        // Create and push event
        authifier
            .publish_event(AuthifierEvent::DeleteSession {
                user_id: self.user_id,
                session_id: self.id,
            })
            .await;

        Ok(())
    }
}
