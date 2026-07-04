use agent_types::{ConversationId, EventSeq, RuntimeEvent};

use crate::event_store::{EventStore, StorageError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayEvent {
    pub seq: EventSeq,
    pub event: RuntimeEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCursor {
    pub conversation_id: ConversationId,
    pub next_seq: Option<EventSeq>,
}

impl ReplayCursor {
    #[must_use]
    pub const fn new(conversation_id: ConversationId) -> Self {
        Self {
            conversation_id,
            next_seq: None,
        }
    }

    #[must_use]
    pub const fn from_sequence(conversation_id: ConversationId, next_seq: EventSeq) -> Self {
        Self {
            conversation_id,
            next_seq: Some(next_seq),
        }
    }

    pub fn collect<S: EventStore>(&self, store: &S) -> Result<Vec<ReplayEvent>, StorageError> {
        store
            .list_by_conversation(self.conversation_id)?
            .into_iter()
            .filter_map(|stored| {
                let seq = stored.event.seq?;
                if self.next_seq.is_some_and(|start| seq < start) {
                    return None;
                }
                Some(Ok(ReplayEvent {
                    seq,
                    event: stored.event,
                }))
            })
            .collect()
    }
}
