use ruse_types::{ConversationId, EventId, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum MemoryScope {
    Conversation(ConversationId),
    Project { workspace_id: WorkspaceId },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySensitivity {
    Public,
    Internal,
    Sensitive,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemorySource {
    pub event_id: Option<EventId>,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub scope: MemoryScope,
    pub text: String,
    pub source: MemorySource,
    pub confidence: u8,
    pub sensitivity: MemorySensitivity,
}
