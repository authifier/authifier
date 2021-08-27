pub use crate::config::Config;
pub use crate::entities::*;
pub use crate::logic::Auth;

pub use mongodb::Client;
pub use rocket::http::{ContentType, Status};
pub use wither::Model;

use mongodb::Database;
use rocket::Route;

pub async fn connect_db() -> Client {
    Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap()
}

pub async fn for_test_with_config(test: &str, config: Config) -> (Database, Auth) {
    let client = connect_db().await;
    let db = client.database(&format!("test::{}", test));
    let auth = Auth::new(db.clone(), config);

    db.drop(None).await.unwrap();
    sync_models(&db).await;

    (db, auth)
}

pub async fn for_test(test: &str) -> Auth {
    for_test_with_config(test, Config::default()).await.1
}

pub async fn bootstrap_rocket_with_auth(
    auth: Auth,
    routes: Vec<Route>,
) -> rocket::local::asynchronous::Client {
    let rocket = rocket::build().manage(auth).mount("/", routes);
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("valid `Rocket`");

    client
}

pub async fn bootstrap_rocket(
    route: &str,
    test: &str,
    routes: Vec<Route>,
) -> rocket::local::asynchronous::Client {
    let auth = for_test(&format!("{}::{}", route, test)).await;
    bootstrap_rocket_with_auth(auth, routes).await
}
