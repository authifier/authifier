/// Send a password reset email
/// POST /account/reset_password

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub email: String,
    pub captcha: Option<String>,
}

/// Responses:
// 204 for success
// Must not allow email enumeration.
