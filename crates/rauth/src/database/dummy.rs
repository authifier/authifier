use crate::{
    models::{Account, Invite, MFATicket, Session},
    Result, Success,
};

use super::{definition::AbstractDatabase, Migration};

#[derive(Clone)]
pub struct DummyDb;

#[async_trait]
impl AbstractDatabase for DummyDb {
    /// Run a database migration
    async fn run_migration(&self, migration: Migration) -> Success {
        todo!("{migration:?}")
    }

    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account> {
        todo!("{id}")
    }

    /// Find account by normalised email
    async fn find_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>> {
        todo!("{normalised_email}")
    }

    /// Find account with active pending email verification
    async fn find_account_with_email_verification(&self, token: &str) -> Result<Account> {
        todo!("{token}")
    }

    /// Find account with active password reset
    async fn find_account_with_password_reset(&self, token: &str) -> Result<Account> {
        todo!("{token}")
    }

    /// Find account with active deletion token
    async fn find_account_with_deletion_token(&self, token: &str) -> Result<Account> {
        todo!("{token}")
    }

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite> {
        todo!("{id}")
    }

    /// Find session by id
    async fn find_session(&self, id: &str) -> Result<Session> {
        todo!("{id}")
    }

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        todo!("{user_id}")
    }

    /// Find sessions by user ids
    async fn find_sessions_with_subscription(&self, user_ids: &[String]) -> Result<Vec<Session>> {
        todo!("{user_ids:?}")
    }

    /// Find session by token
    async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>> {
        todo!("{token}")
    }

    /// Find ticket by token
    async fn find_ticket_by_token(&self, token: &str) -> Result<Option<MFATicket>> {
        todo!("{token}")
    }

    // Save account
    async fn save_account(&self, account: &Account) -> Success {
        todo!("{account:?}")
    }

    /// Save session
    async fn save_session(&self, session: &Session) -> Success {
        todo!("{session:?}")
    }

    /// Save invite
    async fn save_invite(&self, invite: &Invite) -> Success {
        todo!("{invite:?}")
    }

    /// Save ticket
    async fn save_ticket(&self, ticket: &MFATicket) -> Success {
        todo!("{ticket:?}")
    }

    /// Delete session
    async fn delete_session(&self, id: &str) -> Success {
        todo!("{id}")
    }

    /// Delete session
    async fn delete_all_sessions(&self, user_id: &str, ignore: Option<String>) -> Success {
        todo!("{user_id} {ignore:?}")
    }

    /// Delete ticket
    async fn delete_ticket(&self, id: &str) -> Success {
        todo!("{id}")
    }
}
