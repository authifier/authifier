use std::collections::HashMap;

use crate::{Error, Result};

#[derive(Serialize, Deserialize, Clone)]
pub enum Captcha {
    /// Don't require captcha verification
    Disabled,
    /// Use hCaptcha to validate sensitive requests
    #[cfg(feature = "hcaptcha")]
    HCaptcha { secret: String },
}

impl Default for Captcha {
    fn default() -> Captcha {
        Captcha::Disabled
    }
}

impl Captcha {
    /// Check that a given token is valid for the in-use Captcha service
    pub async fn check(&self, token: Option<String>) -> Result<()> {
        match &self {
            #[cfg(feature = "hcaptcha")]
            Captcha::HCaptcha { secret } => {
                if let Some(token) = token {
                    let mut map = HashMap::new();
                    map.insert("secret", secret.clone());
                    map.insert("response", token);

                    let client = reqwest::Client::new();
                    if let Ok(response) = client
                        .post("https://hcaptcha.com/siteverify")
                        .form(&map)
                        .send()
                        .await
                    {
                        #[derive(Serialize, Deserialize)]
                        struct CaptchaResponse {
                            success: bool,
                        }

                        let result: CaptchaResponse =
                            response.json().await.map_err(|_| Error::CaptchaFailed)?;

                        if result.success {
                            Ok(())
                        } else {
                            Err(Error::CaptchaFailed)
                        }
                    } else {
                        Err(Error::CaptchaFailed)
                    }
                } else {
                    Err(Error::CaptchaFailed)
                }
            }
            Captcha::Disabled => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Captcha, Error};

    #[async_std::test]
    async fn it_accepts_if_no_captcha_service() {
        let captcha = Captcha::Disabled;
        assert_eq!(captcha.check(None).await, Ok(()));
        assert_eq!(captcha.check(Some("token".to_string())).await, Ok(()));
    }

    #[async_std::test]
    async fn it_accepts_if_hcaptcha() {
        let captcha = Captcha::HCaptcha {
            secret: "0x0000000000000000000000000000000000000000".to_string(),
        };

        assert_eq!(
            captcha
                .check(Some("20000000-aaaa-bbbb-cccc-000000000002".to_string()))
                .await,
            Ok(())
        );
    }

    #[async_std::test]
    async fn it_rejects_if_hcaptcha_response_is_invalid() {
        let captcha = Captcha::HCaptcha {
            secret: "0x0000000000000000000000000000000000000000".to_string(),
        };

        assert_eq!(
            captcha
                .check(Some("b0000000-aaaa-bbbb-cccc-000000000003".to_string()))
                .await,
            Err(Error::CaptchaFailed)
        );
    }

    #[async_std::test]
    async fn it_rejects_if_no_token() {
        assert_eq!(
            Captcha::HCaptcha {
                secret: "".to_string(),
            }
            .check(None)
            .await,
            Err(Error::CaptchaFailed)
        );
    }
}
