use std::io::Cursor;

use rocket::{http::ContentType, response::Result, Response};

#[get("/reset/<_token>")]
pub async fn reset_password_form(_token: String) -> Result<'static> {
    let body = include_str!("reset_form.html");

    Response::build()
        .header(ContentType::HTML)
        .sized_body(body.len(), Cursor::new(body))
        .ok()
}
