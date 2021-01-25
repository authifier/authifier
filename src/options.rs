pub struct Options {
    pub verified_uri: String,
    pub base_url: String,
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

impl Options {
    pub fn new() -> Options {
        Options {
            verified_uri: "https://example.com".to_string(),
            base_url: "https://example.com".to_string(),
        }
    }

    pub fn verified_uri(self, verified_uri: String) -> Options {
        Options {
            verified_uri,
            ..self
        }
    }

    pub fn base_url(self, base_url: String) -> Options {
        Options { base_url, ..self }
    }
}
