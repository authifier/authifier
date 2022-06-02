<p align="center">
  <img src="banner.png" height="180px" />
</p>

## Errata Notice

Currently, `rocket` is locked to `0.5.3-rc.1` because `rocket_okapi` is not up to date yet.

## Goals

- Prevent user enumeration.

  All routes should be protected against user enumeration.

- Always confirm any change to security settings using two-factor method if available.
- Prevent phishing attacks.

## Usage

Getting started is very simple, create a new instance of the Auth struct and mount it on to Rocket.

```rust
use mongodb::Client;
use rocket;
use rauth;

#[tokio::main]
async fn main() {
  let client = Client::with_uri_str("mongodb://localhost:27017/")
    .await.unwrap();

  // Pick a suitable collection, make sure you set it up correctly
  // as written below in "Database Migrations".
  let col = client.database("rauth").collection("accounts");

  // Set any options, such as the public base URL or your email
  // verification options.
  let options = rauth::options::Options::new();

  // Create a new instance of the Auth object.
  let auth = rauth::auth::Auth::new(col, options);
  rocket::ignite()
    .manage(auth) // Mount rAuth state.
    .mount("/", rauth::routes::routes()) // Mount rAuth routes.
    .launch()
    .await
    .unwrap();
}
```

## How does rAuth work?

rAuth uses email / password combinations to authenticate users and nothing else, this might not be what you're looking for but I personally prefer this format.

- If you need usernames, you need to handle this on your end.

When a user signs in, a new session is created, every single device a user logs in on has a unique session.

- This means a user can then log themselves out of old sessions or otherwise see where they are logged in.

![Example from Revolt App](https://img.insrt.uk/xexu7/daLinuSa38.png/raw)

Internally rAuth stores emails with and without special characters, `+.`.

- This means we can support plus signing without allowing the same email to sign up multiple times.
  - For example, `inbox+a@example.com` and `inbox+b@example.com` are treated as equal.
  - But since we are still storing the original email, we still send them marked with the user's sign.
- In the case of Gmail, all emails with dots are forwarded to those without them, this can lead to some [unfortunate situations](https://jameshfisher.com/2018/04/07/the-dots-do-matter-how-to-scam-a-gmail-user/).
  - Generally, we treat all emails with dots as their non-dot counterpart when checking if an email exists.
  - This may inconvenience some users but I would rather avoid situations like above or duplicate accounts.
- When logging in, the email given is checked against the original email and nothing else.

## Database Migrations

You need to manage the database migrations yourself.

#### Creating the database.

rAuth needs a collection and two indexes to operate optimally.

```rust
db.create_collection("accounts", None);
db.run_command(
  doc! {
    "createIndexes": "accounts",
    "indexes": [
      {
        "key": {
          "email": 1
        },
        "name": "email",
        "unique": true,
        "collation": {
          "locale": "en",
          "strength": 2
        }
      },
      {
        "key": {
          "email_normalised": 1
        },
        "name": "email_normalised",
        "unique": true,
        "collation": {
          "locale": "en",
          "strength": 2
        }
      }
    ]
  },
  None,
);
```

### Migrations

Currently no migrations are available, but they will be added here as needed.
