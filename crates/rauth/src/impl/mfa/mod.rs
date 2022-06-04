use crate::{
    models::{totp::Totp, MultiFactorAuthentication},
    Error, Result, Success,
};

mod totp;

impl MultiFactorAuthentication {
    // Generate new recovery codes
    pub fn generate_recovery_codes(&mut self) {
        static ALPHABET: [char; 32] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z',
        ];

        let mut codes = vec![];
        for _ in 1..=10 {
            codes.push(format!(
                "{}-{}",
                nanoid!(5, &ALPHABET),
                nanoid!(5, &ALPHABET)
            ));
        }

        self.recovery_codes = codes;
    }

    // Generate new TOTP secret
    pub async fn generate_totp_secret(&mut self) -> Success {
        if let Totp::Enabled { .. } = self.totp_token {
            return Err(Error::OperationFailed);
        }

        let secret: [u8; 10] = rand::random();
        let secret = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret);

        self.totp_token = Totp::Pending { secret };

        Ok(())
    }

    // Generate a TOTP code from secret
    pub fn generate_totp_code(&self) -> Result<String> {
        if let Totp::Enabled { secret } = &self.totp_token {
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
