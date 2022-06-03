use crate::{
    models::{Account, Invite, Session},
    Result, Success,
};

use super::definition::AbstractDatabase;

pub struct DummyDb;

#[async_trait]
impl AbstractDatabase for DummyDb {
    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account> {
        todo!()
    }

    /// Find account by normalised email
    async fn find_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>> {
        todo!()
    }

    // FindOneOptions::builder()
    // .collation(Collation::builder().locale("en").strength(2).build())
    // .build(),

    /// Find account with active pending email verification
    async fn find_account_with_email_verification(&self, token: &str) -> Result<Account> {
        todo!()
    }

    /// Find account with active password reset
    async fn find_account_with_password_reset(&self, token: &str) -> Result<Account> {
        todo!()
    }

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite> {
        todo!()
    }

    /// Find session by id
    async fn find_session(&self, id: &str) -> Result<Session> {
        todo!()
    }

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        todo!()
    }

    /// Find session by token
    async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>> {
        todo!()
    }

    // Save account
    async fn save_account(&self, account: &Account) -> Success {
        todo!()
    }

    /// Save session
    async fn save_session(&self, session: &Session) -> Success {
        todo!()
    }

    /// Save invite
    async fn save_invite(&self, invite: &Invite) -> Success {
        todo!()
    }

    /// Delete session
    async fn delete_session(&self, id: &str) -> Success {
        todo!()
    }

    /// Delete session
    async fn delete_all_sessions(&self, user_id: &str, ignore: Option<String>) -> Success {
        todo!()
        /*
            let mut update = doc! {
            "user_id": session.user_id
        };

        if !revoke_self {
            update.insert(
                "_id",
                doc! {
                    "$ne": session.id.unwrap()
                },
            );
        } */
    }
}
