#[derive(Debug, Serialize, Deserialize)]
pub struct Invite {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
}
