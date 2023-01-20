pub use mongodb::Client;
pub use rauth::{
    config::*, database::MongoDb, models::totp::*, models::*, Config, Database, Error, Migration,
    RAuth, RAuthEvent, Result,
};
pub use rocket::http::{ContentType, Status};

use rocket::Route;

use async_std::channel::{unbounded, Receiver};

pub async fn connect_db() -> Client {
    Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap()
}

pub async fn test_smtp_config() -> Config {
    Config {
        email_verification: EmailVerificationConfig::Enabled {
            smtp: SMTPSettings {
                from: "noreply@example.com".into(),
                reply_to: Some("support@revolt.chat".into()),
                host: "127.0.0.1".into(),
                port: Some(1025),
                username: "noreply@example.com".into(),
                password: "password_insecure".into(),
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
                deletion: Template {
                    title: "deletion".into(),
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
    // Wait a moment for sendira to catch the email
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;

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

pub async fn for_test_with_config(test: &str, config: Config) -> (RAuth, Receiver<RAuthEvent>) {
    let client = connect_db().await;

    let database = Database::MongoDb(MongoDb(client.database(&format!("test::{}", test))));

    for migration in [
        Migration::WipeAll,
        Migration::M2022_06_03EnsureUpToSpec,
        Migration::M2022_06_09AddIndexForDeletion,
    ] {
        database.run_migration(migration).await.unwrap();
    }

    let (s, r) = unbounded();

    (
        RAuth {
            database,
            config,
            event_channel: Some(s),
        },
        r,
    )
}

pub async fn for_test(test: &str) -> (RAuth, Receiver<RAuthEvent>) {
    for_test_with_config(test, Config::default()).await
}

pub async fn for_test_authenticated_with_config(
    test: &str,
    config: Config,
) -> (RAuth, Session, Account, Receiver<RAuthEvent>) {
    let (rauth, receiver) = for_test_with_config(test, config).await;

    let account = Account::new(
        &rauth,
        "email@revolt.chat".into(),
        "password_insecure".into(),
        false,
    )
    .await
    .unwrap();

    // clear this event
    receiver.try_recv().expect("an event");

    let session = account
        .create_session(&rauth, "my session".into())
        .await
        .unwrap();

    // clear this event
    receiver.try_recv().expect("an event");

    (rauth, session, account, receiver)
}

pub async fn for_test_authenticated(test: &str) -> (RAuth, Session, Account, Receiver<RAuthEvent>) {
    for_test_authenticated_with_config(test, Config::default()).await
}

pub async fn bootstrap_rocket_with_auth(
    rauth: RAuth,
    routes: Vec<Route>,
) -> rocket::local::asynchronous::Client {
    let rocket = rocket::build().manage(rauth).mount("/", routes);
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("valid `Rocket`");

    client
}

pub async fn bootstrap_rocket(
    route: &str,
    test: &str,
    routes: Vec<Route>,
) -> (rocket::local::asynchronous::Client, Receiver<RAuthEvent>) {
    let (rauth, receiver) = for_test(&format!("{}::{}", route, test)).await;
    (bootstrap_rocket_with_auth(rauth, routes).await, receiver)
}
