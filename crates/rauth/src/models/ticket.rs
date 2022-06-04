/// Multi-factor auth ticket
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct MFATicket {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// Account Id
    pub account_id: String,

    /// Unique Token
    pub token: String,

    /// Whether this ticket has been validated
    pub validated: bool,
}

/// Ticket which is guaranteed to be valid for use
///
/// If used in a Rocket guard, it will be consumed on match
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatedTicket(pub MFATicket);

/// Ticket which is guaranteed to not be valid for use
#[derive(Debug, Serialize, Deserialize)]
pub struct UnvalidatedTicket(pub MFATicket);
