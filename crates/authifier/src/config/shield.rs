use std::collections::HashMap;

use crate::{Error, Result};

#[derive(Serialize, Deserialize, Clone)]
pub enum Shield {
    /// Disable Authifier Shield
    Disabled,

    /// Use Authifier Shield to block malicious actors
    #[cfg(feature = "shield")]
    Enabled {
        /// API key found on your dashboard
        api_key: String,
        /// Whether to always fail if HTTP request fails
        strict: bool,
    },
}

impl Default for Shield {
    fn default() -> Shield {
        Shield::Disabled
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ShieldValidationInput {
    /// Remote user IP
    pub ip: Option<String>,

    /// User provided email
    pub email: Option<String>,

    /// Request headers
    pub headers: Option<HashMap<String, String>>,

    /// Skip alerts and monitoring for this request
    pub dry_run: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether this request was blocked
    blocked: bool,

    /// Reasons for the request being blocked
    reasons: Vec<String>,
}

impl Shield {
    /// Validate a given request
    pub async fn validate(&self, input: ShieldValidationInput) -> Result<()> {
        match &self {
            #[cfg(feature = "shield")]
            Shield::Enabled { api_key, strict } => {
                let client = reqwest::Client::new();
                if let Ok(response) = client
                    .post("https://shield.authifier.com/validate")
                    .json(&input)
                    .header("Authorization", api_key)
                    .send()
                    .await
                {
                    let result: ValidationResult =
                        response.json().await.map_err(|_| Error::InternalError)?;

                    if result.blocked {
                        Err(Error::BlockedByShield)
                    } else {
                        Ok(())
                    }
                } else if *strict {
                    Err(Error::InternalError)
                } else {
                    Ok(())
                }
            }
            Shield::Disabled => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Shield;

    #[async_std::test]
    async fn it_accepts_if_no_shield_service() {
        let shield = Shield::Disabled;
        assert_eq!(shield.validate(Default::default()).await, Ok(()));
    }
}
