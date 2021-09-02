/// Change account email.
/// PATCH /account/email

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub email: String,
    pub current_password: String,
}

/// Responses:
// 204 for success
