use crate::{
    models::{Account, Invite, Session},
    Result, Success,
};

#[async_trait]
pub trait AbstractDatabase: std::marker::Sync {
    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account>;

    /// Find account by normalised email
    async fn find_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>>;

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite>;

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>>;

    /// Find session by token
    async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>>;

    // Insert new account
    async fn insert_account(&self, account: &Account) -> Success;

    // Save account
    async fn save_account(&self, account: &Account) -> Success;

    /// Mark invite as used
    async fn use_invite(&self, id: &str, user_id: &str) -> Success;
}
