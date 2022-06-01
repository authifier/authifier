#[derive(Serialize, Deserialize)]
pub enum Captcha {
    /// Don't require captcha verification
    Disabled,
    /// Use hCaptcha to validate sensitive requests
    HCaptcha { secret: String },
}

impl Default for Captcha {
    fn default() -> Captcha {
        Captcha::Disabled
    }
}
