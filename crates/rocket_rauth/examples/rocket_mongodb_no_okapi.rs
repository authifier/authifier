//! Run example with `cargo run --example rocket_mongodb_no_okapi --features example`

#[macro_use]
extern crate rocket;

#[cfg(feature = "example")]
#[launch]
async fn rocket() -> _ {
    use mongodb::{options::ClientOptions, Client};
    use rauth::database::MongoDb;
    use rauth::Migration;

    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Valid connection URL");

    let client = Client::with_options(client_options).expect("MongoDB server");
    let database = rauth::Database::MongoDb(MongoDb(client.database("rauth")));

    for migration in [Migration::WipeAll, Migration::M2022_06_03EnsureUpToSpec] {
        database.run_migration(migration).await.unwrap();
    }

    let rauth = rauth::RAuth {
        database,
        ..Default::default()
    };

    rocket::build()
        .manage(rauth)
        .mount("/auth/account", rocket_rauth::routes::account::routes().0)
        .mount("/auth/session", rocket_rauth::routes::session::routes().0)
        .mount("/auth/mfa", rocket_rauth::routes::mfa::routes().0)
}

#[cfg(not(feature = "example"))]
fn main() {
    panic!("Enable `example` feature to run this example!");
}
