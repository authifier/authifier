/// Time-based one-time password configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Totp {
    /// Disabled
    Disabled,
    /// Waiting for user activation
    Pending { secret: String },
    /// Required on account
    Enabled { secret: String },
}
