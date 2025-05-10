use std::collections::HashSet;

use sha1::Digest;

use crate::{Error, Result};

#[derive(Default, Serialize, Deserialize, Clone)]
pub enum PasswordScanning {
    /// Disable password scanning
    #[cfg_attr(not(feature = "pwned100k"), default)]
    None,
    /// Use a custom password list
    Custom { passwords: HashSet<String> },
    /// Block the top 100k passwords from HIBP
    #[cfg(feature = "pwned100k")]
    #[default]
    Top100k,
    /// easypwned locally-hosted HIBP database API
    #[cfg(feature = "easypwned")]
    EasyPwned { endpoint: String },
    /// Use the Have I Been Pwned? API
    #[cfg(feature = "have_i_been_pwned")]
    HIBP { api_key: String },
}

#[cfg(feature = "pwned100k")]
lazy_static! {
    /// Top 100k compromised passwords
    static ref TOP_100K_COMPROMISED: HashSet<String> = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/pwned100k.txt"))
        .split('\n')
        .map(|x| x.into())
        .collect();
}

impl PasswordScanning {
    /// Check whether a password can be used
    pub async fn assert_safe(&self, password: &str) -> Result<()> {
        // Make sure the password is long enough.
        if password.len() < 8 {
            return Err(Error::ShortPassword);
        }

        // Check against password lists.
        match self {
            PasswordScanning::None => Ok(()),
            PasswordScanning::Custom { passwords } => {
                if passwords.contains(password) {
                    Err(Error::CompromisedPassword)
                } else {
                    Ok(())
                }
            }
            #[cfg(feature = "easypwned")]
            PasswordScanning::EasyPwned { endpoint } => {
                let mut hasher = sha1::Sha1::new();
                hasher.update(password);
                let pwd_hash = hasher.finalize();

                #[derive(Deserialize)]
                struct EasyPwnedResult {
                    secure: bool,
                }

                let result = match reqwest::get(format!("{endpoint}/hash/{pwd_hash:#02x}")).await {
                    Ok(response) => match response.json::<EasyPwnedResult>().await {
                        Ok(result) => Ok(result.secure),
                        Err(_) => Err(Error::InternalError),
                    },
                    Err(_) => Err(Error::InternalError),
                };

                match result {
                    Ok(true) => Ok(()),
                    // todo: report Err(_) case; ideally we merge into Revolt backend at some point, at that point use create_internal_error! macro
                    _ => Err(Error::CompromisedPassword),
                }
            }
            #[cfg(feature = "pwned100k")]
            PasswordScanning::Top100k => {
                if TOP_100K_COMPROMISED.contains(password) {
                    Err(Error::CompromisedPassword)
                } else {
                    Ok(())
                }
            }
            #[cfg(feature = "have_i_been_pwned")]
            PasswordScanning::HIBP { .. } => {
                unimplemented!("Have I Been Pwned? API is not supported yet.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::PasswordScanning;

    use std::collections::HashSet;

    #[async_std::test]
    async fn it_accepts_any_passwords() {
        let passwords = PasswordScanning::None;
        assert_eq!(passwords.assert_safe("example123").await, Ok(()));
    }

    #[async_std::test]
    async fn it_accepts_some_passwords() {
        let passwords = PasswordScanning::Custom {
            passwords: HashSet::from(["abc".to_string()]),
        };

        assert_eq!(passwords.assert_safe("example123").await, Ok(()));
    }

    #[async_std::test]
    async fn it_rejects_some_passwords() {
        let passwords = PasswordScanning::Custom {
            passwords: HashSet::from(["example123".to_string()]),
        };

        assert_eq!(
            passwords.assert_safe("example123").await,
            Err(Error::CompromisedPassword)
        );

        assert_eq!(
            passwords.assert_safe("short").await,
            Err(Error::ShortPassword)
        );
    }
}
