use crate::models::{Account, Session};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum RAuthEvent {
    CreateAccount { account: Account },
    CreateSession { session: Session },
    DeleteSession { user_id: String, session_id: String },
    DisableAccount { user_id: String },
}
