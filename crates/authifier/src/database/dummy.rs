use crate::{
    models::{
        Account, AuthFlow, Callback, DeletionInfo, EmailVerification, Invite, MFATicket,
        PasswordAuth, Secret, Session,
    },
    Error, Result, Success,
};

use futures::lock::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use super::{definition::AbstractDatabase, Migration};

#[derive(Default, Clone)]
pub struct DummyDb {
    pub accounts: Arc<Mutex<HashMap<String, Account>>>,
    pub callbacks: Arc<Mutex<HashMap<String, Callback>>>,
    pub invites: Arc<Mutex<HashMap<String, Invite>>>,
    pub secrets: Arc<Mutex<HashMap<(), Secret>>>,
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub tickets: Arc<Mutex<HashMap<String, MFATicket>>>,
}

#[async_trait]
impl AbstractDatabase for DummyDb {
    /// Run a database migration
    async fn run_migration(&self, migration: Migration) -> Success {
        println!("skip migration {:?}", migration);
        Ok(())
    }

    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account> {
        let accounts = self.accounts.lock().await;
        accounts.get(id).cloned().ok_or(Error::UnknownUser)
    }

    /// Find account by normalised email
    async fn find_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>> {
        let accounts = self.accounts.lock().await;
        Ok(accounts
            .values()
            .find(|account| account.email_normalised == normalised_email)
            .cloned())
    }

    /// Find account by SSO ID
    async fn find_account_by_sso_id(&self, idp_id: &str, sub_id: &str) -> Result<Option<Account>> {
        todo!()
    }

    /// Find account with active pending email verification
    async fn find_account_with_email_verification(&self, token_to_match: &str) -> Result<Account> {
        let accounts = self.accounts.lock().await;
        accounts
            .values()
            .find(|account| match &account.verification {
                EmailVerification::Pending { token, .. }
                | EmailVerification::Moving { token, .. } => token == token_to_match,
                _ => false,
            })
            .cloned()
            .ok_or(Error::InvalidToken)
    }

    /// Find account with active password reset
    async fn find_account_with_password_reset(&self, token: &str) -> Result<Account> {
        let accounts = self.accounts.lock().await;
        accounts
            .values()
            .find(|account| {
                if let AuthFlow::Password(PasswordAuth {
                    password_reset: Some(reset),
                    ..
                }) = &account.auth_flow
                {
                    reset.token == token
                } else {
                    false
                }
            })
            .cloned()
            .ok_or(Error::InvalidToken)
    }

    /// Find account with active deletion token
    async fn find_account_with_deletion_token(&self, token_to_match: &str) -> Result<Account> {
        let accounts = self.accounts.lock().await;
        accounts
            .values()
            .find(|account| {
                if let Some(DeletionInfo::WaitingForVerification { token, .. }) = &account.deletion
                {
                    token == token_to_match
                } else {
                    false
                }
            })
            .cloned()
            .ok_or(Error::InvalidToken)
    }

    /// Find callback by id
    async fn find_callback(&self, id: &str) -> Result<Callback> {
        let callbacks = self.callbacks.lock().await;
        callbacks.get(id).cloned().ok_or(Error::InvalidState)
    }

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite> {
        let invites = self.invites.lock().await;
        invites.get(id).cloned().ok_or(Error::InvalidInvite)
    }

    /// Find session by id
    async fn find_session(&self, id: &str) -> Result<Session> {
        let sessions = self.sessions.lock().await;
        sessions.get(id).cloned().ok_or(Error::UnknownUser)
    }

    /// Find secret
    async fn find_secret(&self) -> Result<Secret> {
        let secrets = self.secrets.lock().await;

        match secrets.get(&()) {
            Some(secret) => Ok(secret.clone()),
            None => {
                let secret = Secret::new();

                self.save_secret(&secret).await.map(|_| secret)
            }
        }
    }

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions
            .values()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect())
    }

    /// Find sessions by user ids
    async fn find_sessions_with_subscription(&self, user_ids: &[String]) -> Result<Vec<Session>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions
            .values()
            .filter(|session| session.subscription.is_some() && user_ids.contains(&session.id))
            .cloned()
            .collect())
    }

    /// Find session by token
    async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions
            .values()
            .find(|session| session.token == token)
            .cloned())
    }

    /// Find ticket by token
    async fn find_ticket_by_token(&self, token: &str) -> Result<Option<MFATicket>> {
        let tickets = self.tickets.lock().await;
        Ok(tickets
            .values()
            .find(|ticket| ticket.token == token)
            .cloned())
    }

    // Save account
    async fn save_account(&self, account: &Account) -> Success {
        let mut accounts = self.accounts.lock().await;
        accounts.insert(account.id.to_string(), account.clone());
        Ok(())
    }

    // Save callback
    async fn save_callback(&self, callback: &Callback) -> Success {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.insert(callback.id.to_string(), callback.clone());
        Ok(())
    }

    /// Save secret
    async fn save_secret(&self, secret: &Secret) -> Success {
        let mut secrets = self.secrets.lock().await;
        secrets.insert((), secret.clone());
        Ok(())
    }

    /// Save session
    async fn save_session(&self, session: &Session) -> Success {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.id.to_string(), session.clone());
        Ok(())
    }

    /// Save invite
    async fn save_invite(&self, invite: &Invite) -> Success {
        let mut invites = self.invites.lock().await;
        invites.insert(invite.id.to_string(), invite.clone());
        Ok(())
    }

    /// Save ticket
    async fn save_ticket(&self, ticket: &MFATicket) -> Success {
        let mut tickets = self.tickets.lock().await;
        tickets.insert(ticket.id.to_string(), ticket.clone());
        Ok(())
    }

    /// Delete callback
    async fn delete_callback(&self, id: &str) -> Success {
        let mut callbacks = self.callbacks.lock().await;
        if callbacks.remove(id).is_some() {
            Ok(())
        } else {
            Err(Error::InvalidState)
        }
    }

    /// Delete session
    async fn delete_session(&self, id: &str) -> Success {
        let mut sessions = self.sessions.lock().await;
        if sessions.remove(id).is_some() {
            Ok(())
        } else {
            Err(Error::InvalidSession)
        }
    }

    /// Delete session
    async fn delete_all_sessions(&self, user_id: &str, ignore: Option<String>) -> Success {
        let mut sessions = self.sessions.lock().await;
        sessions.retain(|_, session| {
            if session.user_id == user_id {
                if let Some(ignore) = &ignore {
                    ignore == &session.id
                } else {
                    false
                }
            } else {
                true
            }
        });

        Ok(())
    }

    /// Delete ticket
    async fn delete_ticket(&self, id: &str) -> Success {
        let mut tickets = self.tickets.lock().await;
        if tickets.remove(id).is_some() {
            Ok(())
        } else {
            Err(Error::InvalidToken)
        }
    }
}
