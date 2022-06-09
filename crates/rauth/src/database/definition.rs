use crate::{
    models::{Account, Invite, MFATicket, Session},
    Result, Success,
};

use super::Migration;

#[async_trait]
pub trait AbstractDatabase: std::marker::Sync {
    /// Run a database migration
    async fn run_migration(&self, migration: Migration) -> Success;

    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account>;

    /// Find account by normalised email
    async fn find_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>>;

    /// Find account with active pending email verification
    async fn find_account_with_email_verification(&self, token: &str) -> Result<Account>;

    /// Find account with active password reset
    async fn find_account_with_password_reset(&self, token: &str) -> Result<Account>;

    /// Find account with active deletion token
    async fn find_account_with_deletion_token(&self, token: &str) -> Result<Account>;

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite>;

    /// Find session by id
    async fn find_session(&self, id: &str) -> Result<Session>;

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>>;

    /// Find sessions by user ids
    async fn find_sessions_with_subscription(&self, user_ids: &[String]) -> Result<Vec<Session>>;

    /// Find session by token
    async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>>;

    /// Find ticket by token
    async fn find_ticket_by_token(&self, token: &str) -> Result<Option<MFATicket>>;

    // Save account
    async fn save_account(&self, account: &Account) -> Success;

    /// Save session
    async fn save_session(&self, session: &Session) -> Success;

    /// Save invite
    async fn save_invite(&self, invite: &Invite) -> Success;

    /// Save ticket
    async fn save_ticket(&self, ticket: &MFATicket) -> Success;

    /// Delete session
    async fn delete_session(&self, id: &str) -> Success;

    /// Delete session
    async fn delete_all_sessions(&self, user_id: &str, ignore: Option<String>) -> Success;

    /// Delete ticket
    async fn delete_ticket(&self, id: &str) -> Success;
}
