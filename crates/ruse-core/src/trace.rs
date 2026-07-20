use ruse_storage::{EventStore, StorageError};
use ruse_types::{ConversationId, RuntimeEventPayload};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceQuery {
    pub conversation_id: ConversationId,
    pub include_tools: bool,
    pub include_policy: bool,
    pub include_checkpoints: bool,
    pub include_final: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceEntry {
    pub seq: u64,
    pub kind: String,
    pub summary: String,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TraceError {
    #[error("storage failed: {0}")]
    Storage(String),
}

pub fn build_trace<S>(store: &S, query: TraceQuery) -> Result<Vec<TraceEntry>, TraceError>
where
    S: EventStore,
{
    let events = store
        .list_by_conversation(query.conversation_id)
        .map_err(storage_error)?;
    let mut entries = Vec::new();

    for stored in events {
        let Some(seq) = stored.event.seq else {
            continue;
        };
        if let Some((kind, summary)) = trace_payload(&stored.event.payload, &query) {
            entries.push(TraceEntry {
                seq: seq.get(),
                kind,
                summary,
            });
        }
    }

    Ok(entries)
}

fn trace_payload(payload: &RuntimeEventPayload, query: &TraceQuery) -> Option<(String, String)> {
    match payload {
        RuntimeEventPayload::ToolCallProposed(call) if query.include_tools => Some((
            "tool_call".to_string(),
            format!(
                "proposed {} {} input {}",
                call.tool_name,
                call.call_id,
                redact_json(&call.input_json)
            ),
        )),
        RuntimeEventPayload::ToolExecutionStarted(started) if query.include_tools => Some((
            "tool_start".to_string(),
            format!(
                "started {} {}{}{}",
                started.tool_name,
                started.call_id,
                started
                    .policy_decision
                    .as_ref()
                    .map(|decision| format!(" policy={decision}"))
                    .unwrap_or_default(),
                started
                    .sandbox_profile
                    .as_ref()
                    .map(|sandbox| format!(" sandbox={sandbox}"))
                    .unwrap_or_default()
            ),
        )),
        RuntimeEventPayload::ToolExecutionCompleted(completed) if query.include_tools => Some((
            "tool_result".to_string(),
            format!(
                "completed {}{}",
                completed.call_id,
                completed
                    .elapsed_ms
                    .map(|elapsed| format!(" elapsed_ms={elapsed}"))
                    .unwrap_or_default()
            ),
        )),
        RuntimeEventPayload::ToolExecutionFailed(failed) if query.include_tools => Some((
            "tool_error".to_string(),
            format!(
                "failed {} {} recoverable={}{}",
                failed.call_id,
                failed.error_code,
                failed.recoverable,
                failed
                    .elapsed_ms
                    .map(|elapsed| format!(" elapsed_ms={elapsed}"))
                    .unwrap_or_default()
            ),
        )),
        RuntimeEventPayload::ToolObservationRecorded(observed) if query.include_tools => Some((
            "tool_observation".to_string(),
            format!("observed {}", observed.call_id),
        )),
        RuntimeEventPayload::PermissionDecisionRecorded(decision) if query.include_policy => {
            Some((
                "permission".to_string(),
                format!("{}: {}", decision.decision, decision.reason),
            ))
        }
        RuntimeEventPayload::CheckpointRecorded(checkpoint) if query.include_checkpoints => Some((
            "checkpoint".to_string(),
            format!(
                "checkpoint {} recorded: {}",
                checkpoint.checkpoint_id, checkpoint.label
            ),
        )),
        RuntimeEventPayload::FinalResponseProduced(response) if query.include_final => Some((
            "final".to_string(),
            format!("final response: {}", response.message),
        )),
        RuntimeEventPayload::TurnCompleted(completed) if query.include_final => Some((
            "turn_completed".to_string(),
            format!(
                "turn completed {:?}: {}",
                completed.final_state, completed.summary
            ),
        )),
        _ => None,
    }
}

fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_secret_key(key) {
                        (key.clone(), Value::String("[redacted]".to_string()))
                    } else {
                        (key.clone(), redact_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_json).collect()),
        _ => value.clone(),
    }
}

fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("secret")
        || key.contains("token")
        || key.contains("password")
        || key.contains("api_key")
        || key.contains("apikey")
}

fn storage_error(error: StorageError) -> TraceError {
    TraceError::Storage(error.to_string())
}
