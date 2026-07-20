use agent_types::{ActorRef, CorrelationId, EventId, RuntimeEvent, RuntimeEventPayload, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const AUDIT_EXPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditExportOptions {
    pub include_diagnostics: bool,
    pub max_payload_bytes: usize,
}

impl Default for AuditExportOptions {
    fn default() -> Self {
        Self {
            include_diagnostics: true,
            max_payload_bytes: 4096,
        }
    }
}

impl AuditExportOptions {
    fn validate(self) -> Result<Self, AuditExportError> {
        if self.max_payload_bytes == 0 {
            return Err(AuditExportError::InvalidMaxPayloadBytes);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditExportBundle {
    pub schema_version: u32,
    pub generated_at: Timestamp,
    pub records: Vec<AuditExportRecord>,
    pub summary: TraceExportSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditExportRecord {
    pub event_id: EventId,
    pub sequence: Option<u64>,
    pub occurred_at: Timestamp,
    pub actor: ActorRef,
    pub correlation_id: CorrelationId,
    pub category: AuditEventCategory,
    pub action: String,
    pub resource: Option<String>,
    pub decision: Option<String>,
    pub summary: String,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventCategory {
    Lifecycle,
    Model,
    Tool,
    Permission,
    External,
    Diagnostic,
    Other,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceExportSummary {
    pub total_events: usize,
    pub audit_records: usize,
    pub tool_events: usize,
    pub permission_events: usize,
    pub lifecycle_events: usize,
    pub model_events: usize,
    pub external_events: usize,
    pub diagnostic_events: usize,
    pub other_events: usize,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AuditExportError {
    #[error("audit export max_payload_bytes must be greater than zero")]
    InvalidMaxPayloadBytes,
}

pub fn export_audit_bundle(
    events: &[RuntimeEvent],
    options: AuditExportOptions,
) -> Result<AuditExportBundle, AuditExportError> {
    export_audit_bundle_at(events, options, Timestamp::now_utc())
}

pub fn export_audit_bundle_at(
    events: &[RuntimeEvent],
    options: AuditExportOptions,
    generated_at: Timestamp,
) -> Result<AuditExportBundle, AuditExportError> {
    let options = options.validate()?;
    let records = events
        .iter()
        .map(|event| export_record(event, options))
        .collect::<Vec<_>>();
    let summary = summarize(events.len(), &records);

    Ok(AuditExportBundle {
        schema_version: AUDIT_EXPORT_SCHEMA_VERSION,
        generated_at,
        records,
        summary,
    })
}

fn export_record(event: &RuntimeEvent, options: AuditExportOptions) -> AuditExportRecord {
    let payload = export_payload(&event.payload);
    let (summary, mut diagnostics) = bounded_summary(&payload.summary, options.max_payload_bytes);
    if !options.include_diagnostics {
        diagnostics.clear();
    }

    AuditExportRecord {
        event_id: event.id,
        sequence: event.seq.map(agent_types::EventSeq::get),
        occurred_at: event.occurred_at,
        actor: event.actor.clone(),
        correlation_id: event.correlation_id,
        category: payload.category,
        action: payload.action,
        resource: payload.resource,
        decision: payload.decision,
        summary,
        diagnostics,
    }
}

struct PayloadExport {
    category: AuditEventCategory,
    action: String,
    resource: Option<String>,
    decision: Option<String>,
    summary: String,
}

fn export_payload(payload: &RuntimeEventPayload) -> PayloadExport {
    match payload {
        RuntimeEventPayload::PermissionDecisionRecorded(decision) => PayloadExport {
            category: AuditEventCategory::Permission,
            action: "permission.decision".to_string(),
            resource: Some(decision.request_id.clone()),
            decision: Some(decision.decision.clone()),
            summary: format!(
                "permission {} for {}: {}",
                decision.decision,
                decision.request_id,
                redact_text(&decision.reason)
            ),
        },
        RuntimeEventPayload::ApprovalRequested(requested) => PayloadExport {
            category: AuditEventCategory::Permission,
            action: "approval.requested".to_string(),
            resource: Some(requested.request_id.to_string()),
            decision: None,
            summary: format!(
                "approval requested {} {} {}: {}",
                requested.action,
                requested.resource_kind,
                requested.resource,
                redact_text(&requested.reason)
            ),
        },
        RuntimeEventPayload::ApprovalResolved(resolved) => PayloadExport {
            category: AuditEventCategory::Permission,
            action: "approval.resolved".to_string(),
            resource: Some(resolved.request_id.to_string()),
            decision: resolved.decision.clone(),
            summary: format!(
                "approval resolved {} status={} reason={}",
                resolved.request_id,
                resolved.status,
                resolved
                    .reason
                    .as_deref()
                    .map(redact_text)
                    .unwrap_or_else(|| "none".to_string())
            ),
        },
        RuntimeEventPayload::ApprovalExpired(expired) => PayloadExport {
            category: AuditEventCategory::Permission,
            action: "approval.expired".to_string(),
            resource: Some(expired.request_id.to_string()),
            decision: Some("expired".to_string()),
            summary: format!(
                "approval expired {}: {}",
                expired.request_id,
                redact_text(&expired.reason)
            ),
        },
        RuntimeEventPayload::ApprovalCancelled(cancelled) => PayloadExport {
            category: AuditEventCategory::Permission,
            action: "approval.cancelled".to_string(),
            resource: Some(cancelled.request_id.to_string()),
            decision: Some("cancelled".to_string()),
            summary: format!(
                "approval cancelled {}: {}",
                cancelled.request_id,
                redact_text(&cancelled.reason)
            ),
        },
        RuntimeEventPayload::ApprovalConsumed(consumed) => PayloadExport {
            category: AuditEventCategory::Permission,
            action: "approval.consumed".to_string(),
            resource: Some(consumed.request_id.to_string()),
            decision: Some("consumed".to_string()),
            summary: format!(
                "approval consumed {} for {} {}",
                consumed.request_id, consumed.tool_name, consumed.call_id
            ),
        },
        RuntimeEventPayload::ToolCallProposed(call) => PayloadExport {
            category: AuditEventCategory::Tool,
            action: "tool.proposed".to_string(),
            resource: Some(call.tool_name.clone()),
            decision: None,
            summary: format!(
                "tool {} proposed {} input {}",
                call.tool_name,
                call.call_id,
                json_summary(&call.input_json)
            ),
        },
        RuntimeEventPayload::ToolExecutionStarted(started) => PayloadExport {
            category: AuditEventCategory::Tool,
            action: "tool.started".to_string(),
            resource: Some(started.tool_name.clone()),
            decision: None,
            summary: format!(
                "tool {} started {}{}{}",
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
        },
        RuntimeEventPayload::ToolExecutionCompleted(completed) => PayloadExport {
            category: AuditEventCategory::Tool,
            action: "tool.completed".to_string(),
            resource: Some(completed.call_id.clone()),
            decision: None,
            summary: format!(
                "tool completed {}{} output {}",
                completed.call_id,
                completed
                    .elapsed_ms
                    .map(|elapsed| format!(" elapsed_ms={elapsed}"))
                    .unwrap_or_default(),
                json_summary(&completed.output_json)
            ),
        },
        RuntimeEventPayload::ToolExecutionFailed(failed) => PayloadExport {
            category: AuditEventCategory::Tool,
            action: "tool.failed".to_string(),
            resource: Some(failed.call_id.clone()),
            decision: None,
            summary: format!(
                "tool failed {} {} recoverable={}{}: {}",
                failed.call_id,
                failed.error_code,
                failed.recoverable,
                failed
                    .elapsed_ms
                    .map(|elapsed| format!(" elapsed_ms={elapsed}"))
                    .unwrap_or_default(),
                redact_text(&failed.message)
            ),
        },
        RuntimeEventPayload::ToolObservationRecorded(observed) => PayloadExport {
            category: AuditEventCategory::Tool,
            action: "tool.observed".to_string(),
            resource: Some(observed.call_id.clone()),
            decision: None,
            summary: format!(
                "tool observed {}: {}",
                observed.call_id,
                redact_text(&observed.observation)
            ),
        },
        RuntimeEventPayload::ModelStreamStarted(started) => PayloadExport {
            category: AuditEventCategory::Model,
            action: "model.stream_started".to_string(),
            resource: Some(started.model.clone()),
            decision: None,
            summary: format!(
                "model stream started provider={} model={} request={}",
                started.provider, started.model, started.request_id
            ),
        },
        RuntimeEventPayload::ModelOutputDelta(delta) => PayloadExport {
            category: AuditEventCategory::Model,
            action: "model.output_delta".to_string(),
            resource: None,
            decision: None,
            summary: format!("model output delta bytes={}", delta.delta.len()),
        },
        RuntimeEventPayload::ModelReasoningSummaryDelta(delta) => PayloadExport {
            category: AuditEventCategory::Model,
            action: "model.reasoning_summary_delta".to_string(),
            resource: None,
            decision: None,
            summary: format!("model reasoning summary delta bytes={}", delta.delta.len()),
        },
        RuntimeEventPayload::ModelStreamCompleted(completed) => PayloadExport {
            category: AuditEventCategory::Model,
            action: "model.stream_completed".to_string(),
            resource: None,
            decision: None,
            summary: format!(
                "model stream completed {:?} output_tokens={}",
                completed.finish_reason, completed.output_token_count
            ),
        },
        RuntimeEventPayload::RuntimeWarning(warning) => PayloadExport {
            category: AuditEventCategory::Diagnostic,
            action: "runtime.warning".to_string(),
            resource: Some(warning.code.clone()),
            decision: None,
            summary: format!(
                "warning {}: {}",
                warning.code,
                redact_text(&warning.message)
            ),
        },
        RuntimeEventPayload::RuntimeError(error) => PayloadExport {
            category: AuditEventCategory::Diagnostic,
            action: "runtime.error".to_string(),
            resource: Some(error.code.clone()),
            decision: None,
            summary: format!("error {}: {}", error.code, redact_text(&error.message)),
        },
        RuntimeEventPayload::McpServerHealthRecorded(_)
        | RuntimeEventPayload::McpToolsDiscovered(_)
        | RuntimeEventPayload::McpPolicyMappingRecorded(_)
        | RuntimeEventPayload::SkillDiscoveredRecorded(_)
        | RuntimeEventPayload::SkillPromptContributionRecorded(_)
        | RuntimeEventPayload::PluginCapabilityValidated(_) => {
            generic_payload(AuditEventCategory::External, payload)
        }
        RuntimeEventPayload::WorkspaceOpened(_)
        | RuntimeEventPayload::SessionStarted(_)
        | RuntimeEventPayload::ConversationStarted(_)
        | RuntimeEventPayload::UserInputReceived(_)
        | RuntimeEventPayload::TurnStarted(_)
        | RuntimeEventPayload::TurnStateChanged(_)
        | RuntimeEventPayload::ContextAssembled(_)
        | RuntimeEventPayload::PromptSnapshotRecorded(_)
        | RuntimeEventPayload::FinalResponseProduced(_)
        | RuntimeEventPayload::TurnCompleted(_)
        | RuntimeEventPayload::TurnFailed(_)
        | RuntimeEventPayload::PlanTodoSnapshotRecorded(_)
        | RuntimeEventPayload::PlanTodoUpdated(_)
        | RuntimeEventPayload::TaskStateChanged(_)
        | RuntimeEventPayload::TaskNotificationRecorded(_)
        | RuntimeEventPayload::CheckpointRecorded(_)
        | RuntimeEventPayload::ReplayStarted(_)
        | RuntimeEventPayload::ReplayCompleted(_) => {
            generic_payload(AuditEventCategory::Lifecycle, payload)
        }
    }
}

fn generic_payload(category: AuditEventCategory, payload: &RuntimeEventPayload) -> PayloadExport {
    let action = payload_type(payload);
    PayloadExport {
        category,
        action: action.clone(),
        resource: None,
        decision: None,
        summary: format!("runtime event {action}"),
    }
}

fn summarize(total_events: usize, records: &[AuditExportRecord]) -> TraceExportSummary {
    let mut summary = TraceExportSummary {
        total_events,
        audit_records: records.len(),
        ..TraceExportSummary::default()
    };

    for record in records {
        match record.category {
            AuditEventCategory::Lifecycle => summary.lifecycle_events += 1,
            AuditEventCategory::Model => summary.model_events += 1,
            AuditEventCategory::Tool => summary.tool_events += 1,
            AuditEventCategory::Permission => summary.permission_events += 1,
            AuditEventCategory::External => summary.external_events += 1,
            AuditEventCategory::Diagnostic => summary.diagnostic_events += 1,
            AuditEventCategory::Other => summary.other_events += 1,
        }
    }

    summary
}

fn bounded_summary(summary: &str, max_payload_bytes: usize) -> (String, Vec<String>) {
    if summary.len() <= max_payload_bytes {
        return (summary.to_string(), Vec::new());
    }

    let mut end = max_payload_bytes;
    while !summary.is_char_boundary(end) {
        end -= 1;
    }
    (
        format!("{}...[truncated]", &summary[..end]),
        vec!["payload.truncated".to_string()],
    )
}

fn payload_type(payload: &RuntimeEventPayload) -> String {
    serde_json::to_value(payload)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn json_summary(value: &Value) -> String {
    serde_json::to_string(&redact_json(value))
        .unwrap_or_else(|_| "\"[unserializable]\"".to_string())
}

fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_secret_like_key(key) {
                        (key.clone(), Value::String("[redacted]".to_string()))
                    } else {
                        (key.clone(), redact_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_json).collect()),
        Value::String(value) if is_secret_like_value(value) => {
            Value::String("[redacted]".to_string())
        }
        _ => value.clone(),
    }
}

fn redact_text(value: &str) -> String {
    if is_secret_like_value(value) {
        "[redacted]".to_string()
    } else {
        value.to_string()
    }
}

fn is_secret_like_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    ["secret", "token", "password", "api_key", "apikey", "key"]
        .iter()
        .any(|needle| key.contains(needle))
}

fn is_secret_like_value(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains(" sk-")
        || value.contains("bearer sk-")
        || value.contains("secret")
        || value.contains("password")
        || value.contains("token")
        || value.contains("begin private key")
}
