#[derive(Serialize, Deserialize)]
pub enum PasswordScanning {
    /// Disable password scanning
    None,
    /// Use a custom password list
    Custom { passwords: Vec<String> },
    /// Block the top 100k passwords from HIBP
    Top100k,
    /// Use the Have I Been Pwned? API
    HIBP { api_key: String },
}

impl Default for PasswordScanning {
    fn default() -> PasswordScanning {
        PasswordScanning::Top100k
    }
}
