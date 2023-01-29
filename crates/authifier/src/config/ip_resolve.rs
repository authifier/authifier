#[derive(Serialize, Deserialize, Clone)]
pub enum ResolveIp {
    /// Use remote IP
    Remote,

    /// Use Cloudflare headers
    Cloudflare,
}

impl Default for ResolveIp {
    fn default() -> ResolveIp {
        ResolveIp::Remote
    }
}
