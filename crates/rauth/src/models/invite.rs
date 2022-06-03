fn is_false(t: &bool) -> bool {
    !t
}

/// Invite ticket
#[derive(Debug, Serialize, Deserialize)]
pub struct Invite {
    /// Invite code
    #[serde(rename = "_id")]
    pub id: String,
    /// Whether this invite ticket has been used
    #[serde(skip_serializing_if = "is_false", default)]
    pub used: bool,
    /// User ID that this invite was claimed by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
}
