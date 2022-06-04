use chrono::{Duration, Utc};

use crate::{
    models::{MFATicket, UnvalidatedTicket, ValidatedTicket},
    Error, RAuth, Result, Success,
};
use std::ops::Deref;

impl MFATicket {
    /// Create a new MFA ticket
    pub async fn new(rauth: &RAuth, account_id: String, validated: bool) -> Result<MFATicket> {
        let ticket = MFATicket {
            id: ulid::Ulid::new().to_string(),
            account_id,
            token: nanoid!(64),
            validated,
        };

        ticket.save(rauth).await?;
        Ok(ticket)
    }

    /// Save model
    pub async fn save(&self, rauth: &RAuth) -> Success {
        rauth.database.save_ticket(self).await
    }

    /// Check if this MFA ticket has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let datetime = ulid::Ulid::from_string(&self.id)
            .expect("Valid `ulid`")
            .datetime()
            // MFA tickets last 5 minutes
            .checked_add_signed(Duration::minutes(5))
            .expect("checked add signed");

        now > datetime
    }

    /// Claim and remove this MFA ticket
    pub async fn claim(&self, rauth: &RAuth) -> Success {
        if self.is_expired() {
            return Err(Error::InvalidToken);
        }

        rauth.database.delete_ticket(&self.id).await
    }
}

impl Deref for ValidatedTicket {
    type Target = MFATicket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UnvalidatedTicket {
    type Target = MFATicket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
