/// Web Push subscription
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct WebPushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

/// Session information
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Session {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// User Id
    pub user_id: String,

    /// Session token
    pub token: String,

    /// Display name
    pub name: String,

    /// When the session was last logged in (iso8601 timestamp)
    pub last_seen: String,

    /// What is the session origin?
    /// This could be used to differentiate sessions that come from staging/test vs prod, etc.
    /// Authifier will set this to None by default. The application must fill it in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,

    /// Web Push subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription: Option<WebPushSubscription>,
}
