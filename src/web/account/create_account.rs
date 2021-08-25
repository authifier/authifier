/// Create a new account
/// POST /account/create

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub email: String,
    pub password: String,
    pub invite: Option<String>,
    pub captcha: Option<String>,
}

/// Responses:
// 204 for success
// Must not allow email enumeration.
// If an email is already registered, send a password reset link and pretend we succeeded.
