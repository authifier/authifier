//! Run example with `cargo run --example rocket_mongodb --features example`

use okapi::openapi3::OpenApi;

#[macro_use]
extern crate rocket;

#[cfg(feature = "example")]
#[launch]
async fn rocket() -> _ {
    use mongodb::{options::ClientOptions, Client};
    use rauth::database::MongoDb;
    use rauth::Migration;
    use rocket_okapi::{mount_endpoints_and_merged_docs, settings::OpenApiSettings};

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

    let mut rocket = rocket::build();
    let settings = OpenApiSettings::default();

    mount_endpoints_and_merged_docs! {
        rocket, "/".to_owned(), settings,
        "/" => (vec![], custom_openapi_spec()),
        "/auth/account" => rocket_rauth::routes::account::routes(),
        "/auth/session" => rocket_rauth::routes::session::routes(),
        "/auth/mfa" => rocket_rauth::routes::mfa::routes(),
    };

    rocket.manage(rauth).mount(
        "/swagger/",
        rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "../openapi.json".to_owned(),
            ..Default::default()
        }),
    )
}

#[cfg(not(feature = "example"))]
fn main() {
    panic!("Enable `example` feature to run this example!");
}

fn custom_openapi_spec() -> OpenApi {
    OpenApi {
        openapi: OpenApi::default_version(),
        ..Default::default()
    }
}
