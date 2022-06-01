#[derive(Serialize, Deserialize)]
pub enum EmailBlockList {
    /// Don't block any emails
    Disabled,
    /// Block a custom list of domains
    Custom { domains: Vec<String> },
    /// Disposable mail list maintained by revolt.chat
    RevoltSourceList,
}

impl Default for EmailBlockList {
    fn default() -> EmailBlockList {
        EmailBlockList::RevoltSourceList
    }
}
