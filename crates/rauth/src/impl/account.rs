use crate::models::Totp;

impl Totp {
    /// Whether TOTP is disabled
    pub fn is_disabled(&self) -> bool {
        matches!(self, Totp::Disabled)
    }
}

impl Default for Totp {
    fn default() -> Totp {
        Totp::Disabled
    }
}
