/// Run example with `cargo run --example main --features async-std-runtime`

use mongodb::Client;
use rauth;
use rocket;

#[async_std::main]
async fn main() {
    let client = Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap();

    let col = client.database("rauth").collection("accounts");
    let options = rauth::options::Options::new()
        .invite_only_collection(client.database("rauth").collection("invites"));

    let auth = rauth::auth::Auth::new(col, options);
    rocket::build()
        .manage(auth)
        .mount("/", rauth::routes::routes())
        .launch()
        .await
        .unwrap();
}
