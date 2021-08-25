/// Change account password.
/// PATCH /account/change_password

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub password: String,
    pub current_password: String,
}

/// Responses:
// 204 for success
