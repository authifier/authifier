use chrono::{Duration, Utc};

use crate::{
    models::{MFATicket, MultiFactorAuthentication, UnvalidatedTicket, ValidatedTicket},
    Authifier, Error, Success,
};
use std::ops::Deref;

impl MFATicket {
    /// Create a new MFA ticket
    pub fn new(account_id: String, validated: bool) -> MFATicket {
        MFATicket {
            id: ulid::Ulid::new().to_string(),
            account_id,
            token: nanoid!(64),
            validated,
            authorised: false,
            last_totp_code: None,
        }
    }

    /// Populate an MFA ticket with valid MFA codes
    pub async fn populate(&mut self, mfa: &MultiFactorAuthentication) {
        self.last_totp_code = mfa.totp_token.generate_code().ok();
    }

    /// Save model
    pub async fn save(&self, authifier: &Authifier) -> Success {
        authifier.database.save_ticket(self).await
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
    pub async fn claim(&self, authifier: &Authifier) -> Success {
        if self.is_expired() {
            return Err(Error::InvalidToken);
        }

        authifier.database.delete_ticket(&self.id).await
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
