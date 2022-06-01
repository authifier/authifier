#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebPushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub user_id: String,
    pub token: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription: Option<WebPushSubscription>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SessionInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl From<Session> for SessionInfo {
    fn from(item: Session) -> Self {
        SessionInfo {
            id: item.id.expect("`id` present"),
            name: item.name,
        }
    }
}
