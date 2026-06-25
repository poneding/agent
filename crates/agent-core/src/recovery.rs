use std::collections::BTreeSet;

use agent_storage::{
    CheckpointStore, EventStore, IdempotencyKey, IdempotencyStore, ReplayCursor, StorageError,
};
use agent_types::{CheckpointId, ConversationId, EventSeq, RuntimeEventPayload, TurnId};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPlan {
    pub conversation_id: ConversationId,
    pub checkpoint_id: Option<CheckpointId>,
    pub replay_from: EventSeq,
    pub pending_call_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryOutcome {
    pub plan: RecoveryPlan,
    pub replayed_event_count: usize,
    pub resumed: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RecoveryError {
    #[error("storage failed: {0}")]
    Storage(String),
    #[error("checkpoint state is corrupt: {0}")]
    CorruptCheckpoint(String),
}

#[derive(Clone, Debug, Default)]
pub struct RecoveryEngine;

impl RecoveryEngine {
    pub fn plan<S>(
        &self,
        store: &S,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> Result<RecoveryPlan, RecoveryError>
    where
        S: CheckpointStore + ?Sized,
    {
        let checkpoint = store
            .latest_checkpoint(conversation_id, turn_id)
            .map_err(storage_error)?;
        if let Some(checkpoint) = checkpoint {
            validate_checkpoint_state(&checkpoint.state_json)?;
            return Ok(RecoveryPlan {
                conversation_id,
                checkpoint_id: Some(checkpoint.checkpoint_id),
                replay_from: next_sequence(checkpoint.event_seq)?,
                pending_call_ids: Vec::new(),
            });
        }

        Ok(RecoveryPlan {
            conversation_id,
            checkpoint_id: None,
            replay_from: EventSeq::new(1).expect("one is a valid replay sequence"),
            pending_call_ids: Vec::new(),
        })
    }

    pub fn recover<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> Result<RecoveryOutcome, RecoveryError>
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let mut plan = self.plan(events, conversation_id, turn_id)?;
        let replayed = ReplayCursor::from_sequence(conversation_id, plan.replay_from)
            .collect(events)
            .map_err(storage_error)?;
        plan.pending_call_ids = pending_call_ids(&replayed, idempotency)?;

        Ok(RecoveryOutcome {
            replayed_event_count: replayed.len(),
            resumed: true,
            plan,
        })
    }
}

fn pending_call_ids<I>(
    replayed: &[agent_storage::ReplayEvent],
    idempotency: &I,
) -> Result<Vec<String>, RecoveryError>
where
    I: IdempotencyStore + ?Sized,
{
    let mut proposed = BTreeSet::new();
    let mut terminal = BTreeSet::new();

    for replay in replayed {
        match &replay.event.payload {
            RuntimeEventPayload::ToolCallProposed(call) => {
                proposed.insert(call.call_id.clone());
            }
            RuntimeEventPayload::ToolExecutionCompleted(completed) => {
                terminal.insert(completed.call_id.clone());
            }
            RuntimeEventPayload::ToolExecutionFailed(failed) => {
                terminal.insert(failed.call_id.clone());
            }
            RuntimeEventPayload::ToolObservationRecorded(observed) => {
                terminal.insert(observed.call_id.clone());
            }
            _ => {}
        }
    }

    let mut pending = Vec::new();
    for call_id in proposed {
        if terminal.contains(&call_id) {
            continue;
        }
        let key = IdempotencyKey::new(call_id.clone()).map_err(storage_error)?;
        if idempotency
            .get_effect(&key)
            .map_err(storage_error)?
            .is_some()
        {
            continue;
        }
        pending.push(call_id);
    }
    Ok(pending)
}

fn validate_checkpoint_state(value: &serde_json::Value) -> Result<(), RecoveryError> {
    if !value.is_object() {
        return Err(RecoveryError::CorruptCheckpoint(
            "checkpoint state must be a JSON object".to_string(),
        ));
    }
    if value
        .get("corrupt")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(RecoveryError::CorruptCheckpoint(
            "checkpoint state is corrupt".to_string(),
        ));
    }
    Ok(())
}

fn next_sequence(seq: EventSeq) -> Result<EventSeq, RecoveryError> {
    EventSeq::new(seq.get() + 1)
        .map_err(|error| RecoveryError::CorruptCheckpoint(error.to_string()))
}

fn storage_error(error: StorageError) -> RecoveryError {
    RecoveryError::Storage(error.to_string())
}
