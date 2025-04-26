use chrono::{Duration, Utc};

use crate::{models::Callback, Authifier, Error, Success};

impl Callback {
    /// Create a new SSO callback
    pub fn new(idp_id: String, redirect_uri: reqwest::Url) -> Self {
        Callback {
            id: ulid::Ulid::new().to_string(),
            idp_id,
            redirect_uri: redirect_uri.to_string(),
            nonce: None,
            code_verifier: None,
        }
    }

    /// Save model
    pub async fn save(&self, authifier: &Authifier) -> Success {
        authifier.database.save_callback(self).await
    }

    /// Check if this SSO callback has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let datetime = ulid::Ulid::from_string(&self.id)
            .expect("Valid `ulid`")
            .datetime()
            // SSO callbacks last 10 minutes
            .checked_add_signed(Duration::minutes(10))
            .expect("checked add signed");

        now > datetime
    }

    /// Claim and remove this SSO callback
    pub async fn claim(&self, authifier: &Authifier) -> Success {
        if self.is_expired() {
            return Err(Error::InvalidToken);
        }

        authifier.database.delete_callback(&self.id).await
    }
}
