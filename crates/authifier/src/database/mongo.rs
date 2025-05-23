use bson::{to_document, DateTime, Document};
use chrono::{Duration, Utc};
use futures::{stream::TryStreamExt, StreamExt};
use mongodb::options::{Collation, CollationStrength, FindOneOptions, UpdateOptions};
use std::{ops::Deref, str::FromStr};
use ulid::Ulid;

use crate::{
    models::{Account, Invite, MFATicket, Session},
    Error, Result, Success,
};

use super::{definition::AbstractDatabase, Migration};

#[derive(Clone)]
pub struct MongoDb(pub mongodb::Database);

impl Deref for MongoDb {
    type Target = mongodb::Database;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl AbstractDatabase for MongoDb {
    /// Run a database migration
    async fn run_migration(&self, migration: Migration) -> Success {
        match migration {
            #[cfg(debug_assertions)]
            Migration::WipeAll => {
                // Drop the entire database
                self.drop().await.unwrap();
            }
            Migration::M2022_06_03EnsureUpToSpec => {
                if self
                    .collection::<Document>("mfa_tickets")
                    .list_index_names()
                    .await
                    .unwrap_or_default()
                    .contains(&"token".to_owned())
                {
                    return Ok(());
                }

                // Make sure all collections exist
                let list = self.list_collection_names().await.unwrap();
                let collections = ["accounts", "sessions", "invites", "mfa_tickets"];

                for name in collections {
                    if !list.contains(&name.to_string()) {
                        self.create_collection(name).await.unwrap();
                    }
                }

                // Setup index for `accounts`
                let col = self.collection::<Document>("accounts");
                col.drop_indexes().await.unwrap();

                self.run_command(doc! {
                    "createIndexes": "accounts",
                    "indexes": [
                        {
                            "key": {
                                "email": 1
                            },
                            "name": "email",
                            "unique": true,
                            "collation": {
                                "locale": "en",
                                "strength": 2
                            }
                        },
                        {
                            "key": {
                                "email_normalised": 1
                            },
                            "name": "email_normalised",
                            "unique": true,
                            "collation": {
                                "locale": "en",
                                "strength": 2
                            }
                        },
                        {
                            "key": {
                                "verification.token": 1
                            },
                            "name": "email_verification"
                        },
                        {
                            "key": {
                                "password_reset.token": 1
                            },
                            "name": "password_reset"
                        }
                    ]
                })
                .await
                .unwrap();

                // Setup index for `sessions`
                let col = self.collection::<Document>("sessions");
                col.drop_indexes().await.unwrap();

                self.run_command(doc! {
                    "createIndexes": "sessions",
                    "indexes": [
                        {
                            "key": {
                                "token": 1
                            },
                            "name": "token",
                            "unique": true
                        },
                        {
                            "key": {
                                "user_id": 1
                            },
                            "name": "user_id"
                        }
                    ]
                })
                .await
                .unwrap();

                // Setup index for `mfa_tickets`
                let col = self.collection::<Document>("mfa_tickets");
                col.drop_indexes().await.unwrap();

                self.run_command(doc! {
                    "createIndexes": "mfa_tickets",
                    "indexes": [
                        {
                            "key": {
                                "token": 1
                            },
                            "name": "token",
                            "unique": true
                        }
                    ]
                })
                .await
                .unwrap();
            }
            Migration::M2022_06_09AddIndexForDeletion => {
                if self
                    .collection::<Document>("accounts")
                    .list_index_names()
                    .await
                    .expect("list of index names")
                    .contains(&"account_deletion".to_owned())
                {
                    return Ok(());
                }

                self.run_command(doc! {
                    "createIndexes": "accounts",
                    "indexes": [
                        {
                            "key": {
                                "deletion.token": 1
                            },
                            "name": "account_deletion"
                        }
                    ]
                })
                .await
                .unwrap();
            }
            Migration::M2025_02_20AddLastSeenToSession => {
                // i had to remove the transaction code which was a lot more sensible
                // but required transactions hence replica sets =(           - insert
                // check commits 2025-05-14 (authifier/authifier) for old code

                loop {
                    #[derive(Deserialize)]
                    struct SessionId {
                        _id: Ulid,
                    }

                    let sessions: Vec<SessionId> = self
                        .collection("sessions")
                        .find(doc! {
                            "$or": [
                                { "last_seen": { "$exists": false } },
                                { "last_seen": "1970-01-01T00:00:00.000Z" }
                            ]
                        })
                        .limit(50_000) // about 400 batches for 2 million
                        .await
                        .expect("Failed to create cursor for sessions!")
                        .map(|doc| doc.expect("id and username"))
                        .collect()
                        .await;

                    if sessions.is_empty() {
                        break;
                    }

                    for session in sessions {
                        let timestamp = iso8601_timestamp::Timestamp::UNIX_EPOCH
                            + iso8601_timestamp::Duration::seconds(
                                session._id.datetime().timestamp(),
                            );

                        self.collection::<Document>("sessions")
                            .update_one(
                                doc! {
                                    "_id": &session._id.to_string(),
                                },
                                doc! {
                                    "$set": {
                                        "last_seen": timestamp.format().to_string()
                                    }
                                },
                            )
                            .await
                            .expect("Failed to update a session.");
                    }
                }
            }
        }

        Ok(())
    }

    /// Find account by id
    async fn find_account(&self, id: &str) -> Result<Account> {
        self.collection("accounts")
            .find_one(doc! {
                "_id": id
            })
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
            .find_one(doc! {
                "email_normalised": normalised_email
            })
            .with_options(
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
            .find_one(doc! {
                "verification.token": token,
                "verification.expiry": {
                    "$gte": DateTime::now().try_to_rfc3339_string().expect("failed to convert to rfc3339 time string")
                }
            })
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
            .find_one(doc! {
                "password_reset.token": token,
                "password_reset.expiry": {
                    "$gte": DateTime::now().try_to_rfc3339_string().expect("failed to convert to rfc3339 time string")
                }
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidToken)
    }

    /// Find account with active deletion token
    async fn find_account_with_deletion_token(&self, token: &str) -> Result<Account> {
        self.collection("accounts")
            .find_one(doc! {
                "deletion.token": token,
                "deletion.expiry": {
                    "$gte": DateTime::now().try_to_rfc3339_string().expect("failed to convert to rfc3339 time string")
                }
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "account",
            })?
            .ok_or(Error::InvalidToken)
    }

    /// Find accounts which are due to be deleted
    async fn find_accounts_due_for_deletion(&self) -> Result<Vec<Account>> {
        self.collection("accounts")
            .find(doc! {
                "deletion.status": "Scheduled",
                "deletion.after": {
                    "$lte": DateTime::now().try_to_rfc3339_string().expect("failed to convert to rfc3339 time string")
                }
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "accounts",
            })?
            .try_collect()
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "collect",
                with: "accounts",
            })
    }

    /// Find invite by id
    async fn find_invite(&self, id: &str) -> Result<Invite> {
        self.collection("invites")
            .find_one(doc! {
                "_id": id
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "invite",
            })?
            .ok_or(Error::InvalidInvite)
    }

    /// Find session by id
    async fn find_session(&self, id: &str) -> Result<Session> {
        self.collection("sessions")
            .find_one(doc! {
                "_id": id
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "session",
            })?
            .ok_or(Error::UnknownUser)
    }

    /// Find sessions by user id
    async fn find_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        self.collection::<Session>("sessions")
            .find(doc! {
                "user_id": user_id
            })
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

    /// Find sessions by user ids
    async fn find_sessions_with_subscription(&self, user_ids: &[String]) -> Result<Vec<Session>> {
        self.collection::<Session>("sessions")
            .find(doc! {
                "user_id": {
                    "$in": user_ids
                },
                "subscription": {
                    "$exists": true
                }
            })
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
            .find_one(doc! {
                "token": token
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "session",
            })?
            .ok_or(Error::UnknownUser)
    }

    /// Find ticket by token
    /// <br>
    /// Ticket is only valid for 1 minute
    async fn find_ticket_by_token(&self, token: &str) -> Result<Option<MFATicket>> {
        let ticket: MFATicket = self
            .collection("mfa_tickets")
            .find_one(doc! {
                "token": token
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "mfa_ticket",
            })?
            .ok_or(Error::InvalidToken)?;

        if let Ok(ulid) = Ulid::from_str(&ticket.id) {
            if (ulid.datetime() + Duration::minutes(1)) > Utc::now() {
                Ok(Some(ticket))
            } else {
                Err(Error::InvalidToken)
            }
        } else {
            Err(Error::InvalidToken)
        }
    }

    // Save account
    async fn save_account(&self, account: &Account) -> Success {
        self.collection::<Account>("accounts")
            .update_one(
                doc! {
                    "_id": &account.id
                },
                doc! {
                    "$set": to_document(account).map_err(|_| Error::DatabaseError {
                        operation: "to_document",
                        with: "account",
                    })?
                },
            )
            .with_options(UpdateOptions::builder().upsert(true).build())
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
                doc! {
                    "$set": to_document(session).map_err(|_| Error::DatabaseError {
                        operation: "to_document",
                        with: "session",
                    })?,
                },
            )
            .with_options(UpdateOptions::builder().upsert(true).build())
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
                doc! {
                    "$set": to_document(invite).map_err(|_| Error::DatabaseError {
                        operation: "to_document",
                        with: "invite",
                    })?,
                },
            )
            .with_options(UpdateOptions::builder().upsert(true).build())
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "upsert_one",
                with: "invite",
            })
            .map(|_| ())
    }

    /// Save ticket
    async fn save_ticket(&self, ticket: &MFATicket) -> Success {
        self.collection::<MFATicket>("mfa_tickets")
            .update_one(
                doc! {
                    "_id": &ticket.id
                },
                doc! {
                    "$set": to_document(ticket).map_err(|_| Error::DatabaseError {
                        operation: "to_document",
                        with: "ticket",
                    })?,
                },
            )
            .with_options(UpdateOptions::builder().upsert(true).build())
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "upsert_one",
                with: "ticket",
            })
            .map(|_| ())
    }

    /// Delete session
    async fn delete_session(&self, id: &str) -> Success {
        self.collection::<Session>("sessions")
            .delete_one(doc! {
                "_id": id
            })
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
            .delete_many(query)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "session",
            })
            .map(|_| ())
    }

    /// Delete ticket
    async fn delete_ticket(&self, id: &str) -> Success {
        self.collection::<MFATicket>("mfa_tickets")
            .delete_one(doc! {
                "_id": id
            })
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "mfa_ticket",
            })
            .map(|_| ())
    }
}
