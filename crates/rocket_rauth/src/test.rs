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
    use crate::config::{EmailVerification, SMTPSettings, Template, Templates};

    Config {
        email_verification: EmailVerification::Enabled {
            smtp: SMTPSettings {
                from: "noreply@example.com".into(),
                reply_to: Some("support@revolt.chat".into()),
                host: "127.0.0.1".into(),
                port: Some(1025),
                username: "noreply@example.com".into(),
                password: "password".into(),
                use_tls: Some(false),
            },
            expiry: Default::default(),
            templates: Templates {
                verify: Template {
                    title: "verify".into(),
                    text: "[[{{url}}]]".into(),
                    url: "".into(),
                    html: None,
                },
                reset: Template {
                    title: "reset".into(),
                    text: "[[{{url}}]]".into(),
                    url: "".into(),
                    html: None,
                },
                welcome: Some(Template {
                    title: "welcome".into(),
                    text: "[[dummy]]".into(),
                    url: "".into(),
                    html: None,
                }),
            },
        },
        ..Default::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mail {
    id: usize,
    recipients_envelope: Vec<String>,
    subject: String,
    source: String,
    pub code: Option<String>,
}

pub async fn assert_email_sendria(mailbox: String) -> Mail {
    let client = reqwest::Client::new();
    let resp = client
        .get("http://localhost:1080/api/messages/")
        .send()
        .await
        .unwrap();

    #[derive(Serialize, Deserialize)]
    struct SendriaResponse {
        data: Vec<Mail>,
    }

    let result: SendriaResponse = resp.json().await.unwrap();
    let mut found = None;
    for mut entry in result.data {
        if entry.recipients_envelope[0] == mailbox {
            client
                .delete(format!("http://localhost:1080/api/messages/{}", &entry.id))
                .send()
                .await
                .unwrap();

            let re = regex::Regex::new(r"\[\[([A-Za-z0-9_-]*)\]\]").unwrap();
            entry.code = Some(re.captures_iter(&entry.source).next().unwrap()[1].to_string());

            found = Some(entry);
        }
    }

    found.unwrap()
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
