use crate::{models::totp::Totp, Error, Result};

impl Totp {
    /// Whether TOTP is disabled
    pub fn is_disabled(&self) -> bool {
        !matches!(self, Totp::Enabled { .. })
    }

    // Generate a TOTP code from secret
    pub fn generate_code(&self) -> Result<String> {
        if let Totp::Enabled { secret } | Totp::Pending { secret } = &self {
            let seconds: u64 = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            Ok(totp_lite::totp_custom::<totp_lite::Sha1>(
                totp_lite::DEFAULT_STEP,
                6,
                secret.as_bytes(),
                seconds,
            ))
        } else {
            Err(Error::OperationFailed)
        }
    }
}

impl Default for Totp {
    fn default() -> Totp {
        Totp::Disabled
    }
}
