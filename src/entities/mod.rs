mod account;
mod invite;
mod session;
mod ticket;

pub use account::*;
pub use invite::*;
pub use session::*;
pub use ticket::*;

pub use wither::bson::doc;
pub use wither::prelude::*;
pub use wither::Model;

pub use futures::StreamExt;

pub async fn sync_models(db: &mongodb::Database) {
    Account::sync(db).await.expect("`Account` model");
    Invite::sync(db).await.expect("`Invite` model");
    Session::sync(db).await.expect("`Session` model");
    MFATicket::sync(db).await.expect("`MFATicket` model");
}
