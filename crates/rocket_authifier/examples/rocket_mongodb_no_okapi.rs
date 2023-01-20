//! Run example with `cargo run --example rocket_mongodb_no_okapi --features example`

#[macro_use]
extern crate rocket;

#[cfg(feature = "example")]
#[launch]
async fn rocket() -> _ {
    use authifier::database::MongoDb;
    use authifier::Migration;
    use mongodb::{options::ClientOptions, Client};

    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Valid connection URL");

    let client = Client::with_options(client_options).expect("MongoDB server");
    let database = authifier::Database::MongoDb(MongoDb(client.database("authifier")));

    for migration in [Migration::WipeAll, Migration::M2022_06_03EnsureUpToSpec] {
        database.run_migration(migration).await.unwrap();
    }

    let authifier = authifier::Authifier {
        database,
        ..Default::default()
    };

    rocket::build()
        .manage(authifier)
        .mount(
            "/auth/account",
            rocket_authifier::routes::account::routes().0,
        )
        .mount(
            "/auth/session",
            rocket_authifier::routes::session::routes().0,
        )
        .mount("/auth/mfa", rocket_authifier::routes::mfa::routes().0)
}

#[cfg(not(feature = "example"))]
fn main() {
    panic!("Enable `example` feature to run this example!");
}
