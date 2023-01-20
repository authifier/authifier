use crate::{models::totp::Totp, Error, Result};

impl Totp {
    /// Whether TOTP information is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Totp::Disabled)
    }

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
                &base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
                    .expect("valid base32 secret"),
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
