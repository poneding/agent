use agent_types::{CheckpointId, ConversationId, EventId, EventSeq, RuntimeEvent};
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StorageError {
    #[error("duplicate event id: {0}")]
    DuplicateEvent(EventId),
    #[error("duplicate checkpoint id: {0}")]
    DuplicateCheckpoint(CheckpointId),
    #[error("event sequence conflict for conversation")]
    SequenceConflict,
    #[error("checkpoint is invalid: {0}")]
    InvalidCheckpoint(String),
    #[error("event payload serialization failed: {0}")]
    Serialization(String),
    #[error("storage backend failed: {0}")]
    Backend(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEvent {
    pub event: RuntimeEvent,
}

pub trait EventStore {
    fn append(&mut self, event: RuntimeEvent) -> Result<StoredEvent, StorageError>;

    fn list_by_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Vec<StoredEvent>, StorageError>;

    fn get(&self, event_id: EventId) -> Result<Option<StoredEvent>, StorageError>;
}

pub(crate) fn assign_sequence(mut event: RuntimeEvent, next: u64) -> RuntimeEvent {
    event.seq = Some(EventSeq::new(next).expect("storage assigns non-zero event sequence"));
    event
}
