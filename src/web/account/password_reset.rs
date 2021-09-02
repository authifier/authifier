/// Confirm a password reset.
/// PATCH /account/reset_password

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub token: String,
    pub password: String,
}

/// Responses:
// 204 for success
