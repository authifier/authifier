use bson::{to_document, Bson, DateTime};
use futures::stream::TryStreamExt;
use mongodb::options::{Collation, CollationStrength, FindOneOptions, UpdateOptions};
use std::ops::Deref;

use crate::{
    models::{Account, Invite, Session},
    Error, Result, Success,
};

use super::definition::AbstractDatabase;

pub struct MongoDb(pub mongodb::Database);

impl Deref for MongoDb {
    type Target = mongodb::Database;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl AbstractDatabase for MongoDb {
    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account> {
        self.collection("accounts")
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::UnknownUser)
    }

    /// Find account by normalised email
    async fn find_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>> {
        self.collection("accounts")
            .find_one(
                doc! {
                    "email_normalised": normalised_email
                },
                FindOneOptions::builder()
                    .collation(
                        Collation::builder()
                            .locale("en")
                            .strength(CollationStrength::Secondary)
                            .build(),
                    )
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })
    }

    /// Find account with active pending email verification
    async fn find_account_with_email_verification(&self, token: &str) -> Result<Account> {
        self.collection("accounts")
            .find_one(
                doc! {
                    "verification.token": token,
                    "verification.expiry": {
                        "$gte": Bson::DateTime(DateTime::now())
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidToken)
    }

    /// Find account with active password reset
    async fn find_account_with_password_reset(&self, token: &str) -> Result<Account> {
        self.collection("accounts")
            .find_one(
                doc! {
                    "password_reset.token": token,
                    "password_reset.expiry": {
                        "$gte": Bson::DateTime(DateTime::now())
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidToken)
    }

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite> {
        self.collection("invites")
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "invite",
            })?
            .ok_or(Error::UnknownUser)
    }

    /// Find session by id
    async fn find_session(&self, id: &str) -> Result<Session> {
        self.collection("sessions")
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "session",
            })?
            .ok_or(Error::UnknownUser)
    }

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        self.collection::<Session>("invites")
            .find(
                doc! {
                    "user_id": user_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "sessions",
            })?
            .try_collect()
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "collect",
                with: "sessions",
            })
    }

    /// Find session by token
    async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>> {
        self.collection("sessions")
            .find_one(
                doc! {
                    "token": token
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "session",
            })?
            .ok_or(Error::UnknownUser)
    }

    // Save account
    async fn save_account(&self, account: &Account) -> Success {
        self.collection::<Account>("accounts")
            .update_one(
                doc! {
                    "_id": &account.id
                },
                to_document(account).map_err(|_| Error::DatabaseError {
                    operation: "to_document",
                    with: "account",
                })?,
                UpdateOptions::builder().build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "upsert_one",
                with: "account",
            })
            .map(|_| ())
    }

    /// Save session
    async fn save_session(&self, session: &Session) -> Success {
        self.collection::<Session>("sessions")
            .update_one(
                doc! {
                    "_id": &session.id
                },
                to_document(session).map_err(|_| Error::DatabaseError {
                    operation: "to_document",
                    with: "session",
                })?,
                UpdateOptions::builder().build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "upsert_one",
                with: "session",
            })
            .map(|_| ())
    }

    /// Save invite
    async fn save_invite(&self, invite: &Invite) -> Success {
        self.collection::<Invite>("invites")
            .update_one(
                doc! {
                    "_id": &invite.id
                },
                to_document(invite).map_err(|_| Error::DatabaseError {
                    operation: "to_document",
                    with: "invite",
                })?,
                UpdateOptions::builder().build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "upsert_one",
                with: "invite",
            })
            .map(|_| ())
    }

    /// Delete session
    async fn delete_session(&self, id: &str) -> Success {
        self.collection::<Session>("sessions")
            .delete_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "session",
            })
            .map(|_| ())
    }

    /// Delete session
    async fn delete_all_sessions(&self, user_id: &str, ignore: Option<String>) -> Success {
        let mut query = doc! {
            "user_id": user_id
        };

        if let Some(id) = ignore {
            query.insert(
                "_id",
                doc! {
                    "$ne": id
                },
            );
        }

        self.collection::<Session>("sessions")
            .delete_many(query, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "session",
            })
            .map(|_| ())
    }
}
