use mongodb::Client;
use rocket;
use rauth;

#[tokio::main]
async fn main() {
    let client = Client::with_uri_str("mongodb://localhost:27017/").await.unwrap();
    let col = client.database("rauth").collection("accounts");
    let auth = rauth::auth::Auth::new(col);
    rauth::routes::mount(rocket::ignite(), "/", auth)
        .launch();
}
