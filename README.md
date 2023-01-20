<p align="center">
  <img src="https://github.com/authifier/authifier/blob/master/assets/Logo_GithubAutoTheme.svg" height="180px" />
</p>

## Goals

- Prevent user enumeration.

  All routes should be protected against user enumeration.

- Always confirm any change to security settings using two-factor method if available.
- Prevent phishing attacks.

## Play around with Authifier API

You can play around with the API by using the provided example and using Swagger:

```bash
# Clone the project
git clone https://github.com/insertish/authifier
cd authifier

# Bring up MongoDB
docker-compose up -d database

# Start the example
cargo run --example rocket_mongodb --features example
```

Now you can navigate to http://localhost:8000/swagger!

## Usage

Getting started is very simple, first add Authifier to your `Cargo.toml`:

```toml
[dependencies]
authifier = { git = "https://github.com/insertish/authifier", features = [ "rocket_impl", "okapi_impl", "async-std-runtime", "database-mongodb" ] }
rocket_authifier = { git = "https://github.com/insertish/authifier" }

# For the example below, you also need:
rocket = { version = "0.5.0-rc.2", default-features = false, features = ["json"] }
mongodb = { version = "2.2.1", default-features = false, features = ["async-std-runtime"] }
```

Then you can create a new instance of Authifier and mount it on to Rocket.

```rust
#[macro_use]
extern crate rocket;

use mongodb::{options::ClientOptions, Client};
use authifier::database::MongoDb;
use authifier::Migration;

#[launch]
async fn rocket() -> _ {
  // Prepare MongoDB configuration
  let client_options = ClientOptions::parse("mongodb://localhost:27017")
    .await
    .expect("Valid connection URL");

  // Connect to MongoDB
  let client = Client::with_options(client_options).expect("MongoDB server");

  // Prepare Authifier database abstraction
  let database = authifier::Database::MongoDb(MongoDb(client.database("authifier")));

  // Run database migrations
  // TODO: you should only run this once and have this as part of your migrations
  // Also keep this up to date with the "migrations" section down below this one.
  database.run_migration(Migration::M2022_06_03EnsureUpToSpec).await.unwrap();

  // Configure Authifier however you need to
  let authifier = authifier::Authifier {
    database,
    ..Default::default()
  };

  // Build your web server as usual...
  rocket::build()
    // Attach the configuration as state
    .manage(authifier)
    // Mount authentication routes
    .mount("/auth/account", rocket_authifier::routes::account::routes().0)
    .mount("/auth/session", rocket_authifier::routes::session::routes().0)
    .mount("/auth/mfa", rocket_authifier::routes::mfa::routes().0)
}
```

## Testing

To test the library, pull up required services:

```bash
# Start MongoDB and Sendria
docker-compose up -d
```

Then you can run the tests:

```bash
# Run cargo test
cargo test --features test

# Or using nextest
cargo nextest run --features test
```

Run a coverage test:

```bash
# Run tarpaulin
cargo tarpaulin --features test --out html
```

## Database Migrations

Migrating the database is easy, you should orchestrate it yourself, ideally you have your own versioned migration system which you can slot changes into.

But, by default, migrations will not run if the system detects that it has probably already been applied.

```rust
use authifier::{ Database, Migration };

// Acquire the database first
let database = Database::[..];

// Then run a specific migration
database.run_migration(Migration::[..]).await.unwrap();
```

The following migrations are available and must be run in order:

| Date       | Migration                   | Description                                                                                          |
| ---------- | --------------------------- | ---------------------------------------------------------------------------------------------------- |
| 2022-06-03 | `M2022_06_03EnsureUpToSpec` | Reset and reconstruct indexes to be fully up to date. This will also create any missing collections. |

## How does Authifier work?

Authifier uses email / password combinations to authenticate users and nothing else, this might not be what you're looking for but I personally prefer this format.

- If you need usernames, you need to handle this on your end.

When a user signs in, a new session is created, every single device a user logs in on has a unique session.

- This means a user can then log themselves out of old sessions or otherwise see where they are logged in.

![Example from Revolt App](https://img.insrt.uk/xexu7/daLinuSa38.png/raw)

Internally Authifier stores emails with and without special characters, `+.`.

- This means we can support plus signing without allowing the same email to sign up multiple times.
  - For example, `inbox+a@example.com` and `inbox+b@example.com` are treated as equal.
  - But since we are still storing the original email, we still send them marked with the user's sign.
- In the case of Gmail, all emails with dots are forwarded to those without them, this can lead to some [unfortunate situations](https://jameshfisher.com/2018/04/07/the-dots-do-matter-how-to-scam-a-gmail-user/).
  - Generally, we treat all emails with dots as their non-dot counterpart when checking if an email exists.
  - This may inconvenience some users but I would rather avoid situations like above or duplicate accounts.
- When logging in, we use the normalised email to find the correct account.
