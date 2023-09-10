/// Time-based one-time password configuration
#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "status")]
pub enum Totp {
    /// Disabled
    #[default]
    Disabled,
    /// Waiting for user activation
    Pending { secret: String },
    /// Required on account
    Enabled { secret: String },
}
