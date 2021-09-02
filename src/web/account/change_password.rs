/// Change account password.
/// PATCH /account/password

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub password: String,
    pub current_password: String,
}

/// Responses:
// 204 for success
