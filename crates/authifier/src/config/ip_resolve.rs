#[derive(Default, Serialize, Deserialize, Clone)]
pub enum ResolveIp {
    /// Use remote IP
    #[default]
    Remote,

    /// Use Cloudflare headers
    Cloudflare,
}
