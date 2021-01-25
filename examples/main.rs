use mongodb::Client;
use rauth;
use rocket;

#[tokio::main]
async fn main() {
    let client = Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap();
    
    let col = client.database("rauth").collection("accounts");
    let options = rauth::options::Options::new();

    let auth = rauth::auth::Auth::new(col, options);
    rocket::ignite()
        .manage(auth)
        .mount("/", rauth::routes::routes())
        .launch()
        .await
        .unwrap();
}
