/// Login to an account
/// POST /session/login

#[derive(Serialize, Deserialize)]
pub enum LoginType {
    Email,
    Password {
        password: String
    },
    SecurityKey {
        challenge: String
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub email: String,

    pub friendly_name: Option<String>,
    pub captcha: Option<String>,

    #[serde(flatten)]
    pub login_type: LoginType
}

/// Responses:
// { token: String } for email/password or email/challenge
// { token: String } for email (will allow email enumeration for email OTP 1FA users, warn users about this)
// { ticket: String, allowed_methods: Method[] } for MFA

// 1. Fetch account
// 2. Verify whether the 1FA method is valid
// 3. Create a session if data is correct
// 4. If MFA is required or requested, create a ticket
