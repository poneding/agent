use ruse_policy::PolicyEngine;
use ruse_storage::{IdempotencyKey, IdempotencyStore, StorageError, ToolEffectRecord};
use ruse_tool::ToolRegistry;
use ruse_types::{
    ActorRef, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, ToolObservationRecorded,
    TurnState,
};
use thiserror::Error;

use crate::{
    RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopError, ToolLoopOutcome, ToolLoopStatus,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdempotentToolOutcome {
    pub tool: ToolLoopOutcome,
    pub replayed: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum IdempotentToolError {
    #[error("tool loop failed: {0}")]
    ToolLoop(String),
    #[error("idempotency storage failed: {0}")]
    Storage(String),
}

#[derive(Clone, Debug, Default)]
pub struct IdempotentToolExecutor {
    tool_loop: ToolLoop,
}

impl IdempotentToolExecutor {
    /// Synchronous path for fixtures / non-async callers.
    pub fn run<P, S>(
        &self,
        ids: RuntimeIds,
        key: IdempotencyKey,
        call: ToolCallRequest,
        registry: &ToolRegistry,
        policy: &P,
        effects: &mut S,
    ) -> Result<IdempotentToolOutcome, IdempotentToolError>
    where
        P: PolicyEngine + ?Sized,
        S: IdempotencyStore + ?Sized,
    {
        let input_hash = input_hash(&call);
        if let Some(record) = effects.get_effect(&key).map_err(storage_error)? {
            if record.input_hash != input_hash {
                return Err(storage_error(StorageError::IdempotencyConflict {
                    key: key.to_string(),
                    existing_hash: record.input_hash,
                    new_hash: input_hash,
                }));
            }
            return Ok(IdempotentToolOutcome {
                tool: replayed_outcome(ids, &record),
                replayed: true,
            });
        }

        let outcome = self
            .tool_loop
            .run(ids, call.clone(), registry, policy)
            .map_err(tool_loop_error)?;
        self.record_completed(ids, key, call, input_hash, outcome, effects)
    }

    /// Async path used by the interactive agent loop: tool execute runs via
    /// `spawn_blocking` so shell/fs work cannot stall streaming.
    pub async fn run_async<P, S>(
        &self,
        ids: RuntimeIds,
        key: IdempotencyKey,
        call: ToolCallRequest,
        registry: &ToolRegistry,
        policy: &P,
        effects: &mut S,
    ) -> Result<IdempotentToolOutcome, IdempotentToolError>
    where
        P: PolicyEngine + ?Sized,
        S: IdempotencyStore + ?Sized,
    {
        let input_hash = input_hash(&call);
        if let Some(record) = effects.get_effect(&key).map_err(storage_error)? {
            if record.input_hash != input_hash {
                return Err(storage_error(StorageError::IdempotencyConflict {
                    key: key.to_string(),
                    existing_hash: record.input_hash,
                    new_hash: input_hash,
                }));
            }
            return Ok(IdempotentToolOutcome {
                tool: replayed_outcome(ids, &record),
                replayed: true,
            });
        }

        let outcome = self
            .tool_loop
            .run_async(ids, call.clone(), registry, policy)
            .await
            .map_err(tool_loop_error)?;
        self.record_completed(ids, key, call, input_hash, outcome, effects)
    }

    fn record_completed<S>(
        &self,
        _ids: RuntimeIds,
        key: IdempotencyKey,
        call: ToolCallRequest,
        input_hash: String,
        outcome: ToolLoopOutcome,
        effects: &mut S,
    ) -> Result<IdempotentToolOutcome, IdempotentToolError>
    where
        S: IdempotencyStore + ?Sized,
    {
        if outcome.status == ToolLoopStatus::Completed
            && let Some(output_json) = completed_output(&outcome)
        {
            effects
                .record_effect(ToolEffectRecord {
                    key,
                    call_id: call.call_id,
                    tool_name: call.tool_name,
                    input_hash,
                    output_json,
                    completed_at: Timestamp::now_utc(),
                })
                .map_err(storage_error)?;
        }

        Ok(IdempotentToolOutcome {
            tool: outcome,
            replayed: false,
        })
    }
}

fn input_hash(call: &ToolCallRequest) -> String {
    let input =
        serde_json::to_string(&call.input_json).unwrap_or_else(|_| call.input_json.to_string());
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv1a(hash, call.tool_name.as_bytes());
    hash = fnv1a(hash, b"\0");
    hash = fnv1a(hash, input.as_bytes());
    format!("fnv1a64:{hash:016x}")
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn completed_output(outcome: &ToolLoopOutcome) -> Option<serde_json::Value> {
    outcome
        .events
        .iter()
        .find_map(|event| match &event.payload {
            RuntimeEventPayload::ToolExecutionCompleted(completed) => {
                Some(completed.output_json.clone())
            }
            _ => None,
        })
}

fn replayed_outcome(ids: RuntimeIds, record: &ToolEffectRecord) -> ToolLoopOutcome {
    let observation = record.output_json.to_string();
    ToolLoopOutcome {
        status: ToolLoopStatus::Completed,
        turn_state: TurnState::Observed,
        observation: Some(observation.clone()),
        pending_approval: None,
        events: vec![RuntimeEvent {
            schema_version: SchemaVersion::current(),
            id: ruse_types::EventId::new(),
            seq: None,
            occurred_at: Timestamp::now_utc(),
            workspace_id: ids.workspace_id,
            conversation_id: ids.conversation_id,
            turn_id: Some(ids.turn_id),
            task_id: None,
            agent_id: None,
            team_id: None,
            actor: ActorRef::system(),
            correlation_id: ids.correlation_id,
            causation_id: None,
            payload: RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
                call_id: record.call_id.clone(),
                observation,
            }),
        }],
    }
}

fn tool_loop_error(error: ToolLoopError) -> IdempotentToolError {
    IdempotentToolError::ToolLoop(error.to_string())
}

fn storage_error(error: StorageError) -> IdempotentToolError {
    IdempotentToolError::Storage(error.to_string())
}
