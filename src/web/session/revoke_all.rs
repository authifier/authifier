/// Revoke all sessions
/// DELETE /session/all

#[derive(Serialize, Deserialize)]
pub struct Data {
    #[default]
    // By default, we won't revoke our session.
    // But a client may choose to do that.
    pub revoke_self: bool,
}

/// Responses:
// 204 for success
