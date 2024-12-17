/// Single sign-on auth callback
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Callback {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// The authorization provider ID.
    pub idp_id: String,

    /// The URI where the end-user will be redirected after authorization.
    pub redirect_uri: String,

    /// A string to mitigate replay attacks.
    pub nonce: Option<String>,

    /// A string to correlate the authorization request to the token request.
    pub code_verifier: Option<String>,
}
