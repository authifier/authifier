/// Secret model
#[derive(Serialize, Deserialize, Clone)]
pub struct Secret(String);

impl Secret {
    pub fn expose(&self) -> &str {
        &*self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secret: String = std::iter::repeat_n('X', self.0.len()).collect();

        f.debug_tuple("Secret").field(&secret).finish()
    }
}
