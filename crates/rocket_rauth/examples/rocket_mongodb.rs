//! Run example with `cargo run --example rocket_mongodb --features example`

#[macro_use]
extern crate rocket;

#[cfg(feature = "example")]
#[launch]
async fn rocket() -> _ {
    use mongodb::{options::ClientOptions, Client};

    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Valid connection URL");

    let client = Client::with_options(client_options).expect("MongoDB server");

    let rauth = rauth::RAuth {
        database: rauth::Database::Mongo(client.database("rauth")),
        ..Default::default()
    };

    rocket::build()
        .manage(rauth)
        .mount("/account", rocket_rauth::routes::account::routes().0)
        .mount("/session", rocket_rauth::routes::session::routes().0)
}

#[cfg(not(feature = "example"))]
fn main() {
    panic!("Enable `example` feature to run this example!");
}
