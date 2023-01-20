use crate::models::{Account, Session};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum AuthifierEvent {
    CreateAccount {
        account: Account,
    },
    CreateSession {
        session: Session,
    },
    DeleteSession {
        user_id: String,
        session_id: String,
    },
    DeleteAllSessions {
        user_id: String,
        exclude_session_id: Option<String>,
    },
}
