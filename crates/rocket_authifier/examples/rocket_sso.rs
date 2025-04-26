//! Run example with `cargo run --example rocket_mongodb --features example`

use revolt_okapi::openapi3::OpenApi;

#[macro_use]
extern crate rocket;

#[cfg(feature = "example")]
#[launch]
async fn rocket() -> _ {
    use authifier::database::DummyDb;
    use authifier::models::Secret;
    use authifier::Migration;
    use revolt_rocket_okapi::{mount_endpoints_and_merged_docs, settings::OpenApiSettings};
    use rocket::figment::providers::{Format as _, Toml};
    use rocket::figment::Figment;

    let database = authifier::database::Database::Dummy(DummyDb::default());

    database.save_secret(&Secret::new()).await.unwrap();

    for migration in [Migration::WipeAll, Migration::M2022_06_03EnsureUpToSpec] {
        database.run_migration(migration).await.unwrap();
    }

    let config = Figment::new()
        .merge(Toml::file("config.toml"))
        .extract()
        .unwrap();

    let authifier = authifier::Authifier {
        database,
        config,
        ..Default::default()
    };

    let mut rocket = rocket::build();
    let settings = OpenApiSettings::default();

    mount_endpoints_and_merged_docs! {
        rocket, "/".to_owned(), settings,
        "/" => (vec![], custom_openapi_spec()),
        "/auth/account" => rocket_authifier::routes::account::routes(),
        "/auth/session" => rocket_authifier::routes::session::routes(),
        "/auth/mfa" => rocket_authifier::routes::mfa::routes(),
        "/auth/sso" => rocket_authifier::routes::sso::routes(),
    };

    rocket.manage(authifier).mount(
        "/swagger/",
        revolt_rocket_okapi::swagger_ui::make_swagger_ui(
            &revolt_rocket_okapi::swagger_ui::SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            },
        ),
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
