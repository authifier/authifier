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

pub async fn test_smtp_config() -> Config {
    dotenv::dotenv().ok();

    use crate::config::{EmailVerification, SMTPSettings, Template, Templates};
    use std::env::var;

    let from = var("SMTP_FROM").unwrap_or("noreply@example.com".into());
    let host = var("SMTP_HOST").unwrap_or("localhost".into());
    let username = var("SMTP_USER").unwrap_or("noreply@example.com".into());
    let password = var("SMTP_PASS").unwrap_or("password".into());
    let use_tls = Some(var("SMTP_USE_TLS").unwrap_or("0".into()) == "0");

    Config {
        email_verification: EmailVerification::Enabled {
            smtp: SMTPSettings {
                from,
                reply_to: Some("support@revolt.chat".into()),
                host,
                port: if var("SMTP_HOST").is_err() {
                    Some(1025)
                } else {
                    None
                },
                username,
                password,
                use_tls,
            },
            expiry: Default::default(),
            templates: Templates {
                verify: Template {
                    title: "Verify your email!".into(),
                    text: "Verify your email here: {{url}}".into(),
                    url: "https://example.com".into(),
                    html: None,
                },
                reset: Template {
                    title: "Reset your password!".into(),
                    text: "Reset your password here: {{url}}".into(),
                    url: "https://example.com".into(),
                    html: None,
                },
                welcome: None,
            },
        },
        ..Default::default()
    }
}

pub async fn for_test_with_config(test: &str, config: Config) -> (Database, Auth) {
    let client = connect_db().await;
    let db = client.database(&format!("test::{}", test));
    let auth = Auth::new(db.clone(), config);

    db.drop(None).await.unwrap();
    sync_models(&db).await;

    (db, auth)
}

pub async fn for_test(test: &str) -> (Database, Auth) {
    for_test_with_config(test, Config::default()).await
}

pub async fn for_test_authenticated_with_config(
    test: &str,
    config: Config,
) -> (Database, Auth, Session, Account) {
    let (db, auth) = for_test_with_config(test, config).await;

    let account = auth
        .create_account("email@example.com".into(), "password".into(), false)
        .await
        .unwrap();

    let session = auth
        .create_session(&account, "my session".into())
        .await
        .unwrap();

    (db, auth, session, account)
}

pub async fn for_test_authenticated(test: &str) -> (Database, Auth, Session, Account) {
    for_test_authenticated_with_config(test, Config::default()).await
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
    let (_, auth) = for_test(&format!("{}::{}", route, test)).await;
    bootstrap_rocket_with_auth(auth, routes).await
}
