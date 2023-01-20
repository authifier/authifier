use std::ops::Deref;

use self::{definition::AbstractDatabase, dummy::DummyDb};

pub mod definition;

#[derive(Debug)]
pub enum Migration {
    M2022_06_03EnsureUpToSpec,
    M2022_06_09AddIndexForDeletion,
    #[cfg(debug_assertions)]
    WipeAll,
}

mod dummy;

#[cfg(feature = "database-mongodb")]
mod mongo;

#[cfg(feature = "database-mongodb")]
pub use mongo::MongoDb;

#[derive(Clone)]
pub enum Database {
    Dummy(DummyDb),
    #[cfg(feature = "database-mongodb")]
    MongoDb(mongo::MongoDb),
}

impl Default for Database {
    fn default() -> Self {
        Self::Dummy(DummyDb)
    }
}

impl Deref for Database {
    type Target = dyn AbstractDatabase;

    fn deref(&self) -> &Self::Target {
        match self {
            Database::Dummy(dummy) => dummy,
            #[cfg(feature = "database-mongodb")]
            Database::MongoDb(mongo) => mongo,
        }
    }
}
