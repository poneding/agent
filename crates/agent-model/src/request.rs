use agent_types::{ConversationId, CorrelationId, TurnId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRequest {
    pub provider: String,
    pub model: String,
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
    pub messages: Vec<String>,
}
