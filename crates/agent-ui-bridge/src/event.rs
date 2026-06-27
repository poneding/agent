use agent_policy::{CapabilityAction, PendingApproval, ResourceKind};
use agent_types::{
    CheckpointId, ClientKind, ConversationId, CorrelationId, EventId, EventSeq, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, TaskId, Timestamp, TurnId, TurnState, WorkspaceTrustLevel,
};
use serde::{Deserialize, Serialize};

use crate::frame::{UiPromptDiagnosticSummary, UiPromptLayerSummary, UiPromptSummary};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiEventEnvelope {
    pub schema_version: SchemaVersion,
    pub event_id: EventId,
    pub seq: Option<EventSeq>,
    pub occurred_at: Timestamp,
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub task_id: Option<TaskId>,
    pub correlation_id: CorrelationId,
    pub payload: UiEventPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiEventPayload {
    Workspace {
        root_path: String,
        trust_level: String,
    },
    Session {
        client_kind: String,
        client_version: String,
    },
    UserMessage {
        message: String,
    },
    TurnState {
        state: TurnState,
        reason: String,
    },
    ModelDelta {
        delta: String,
    },
    FinalResponse {
        message: String,
    },
    ToolCall {
        call_id: String,
        tool_name: String,
    },
    ToolResult {
        call_id: String,
        ok: bool,
        summary: String,
    },
    Permission {
        request_id: String,
        decision: String,
        reason: String,
    },
    ApprovalRequested(Box<UiApprovalRequest>),
    ApprovalResolved {
        request_id: String,
        status: String,
        decision: Option<String>,
        reason: Option<String>,
    },
    Checkpoint {
        checkpoint_id: CheckpointId,
        label: String,
    },
    Diagnostic {
        level: UiDiagnosticLevel,
        code: String,
        message: String,
        recoverable: bool,
    },
    PromptSummary(UiPromptSummary),
    Progress {
        message: String,
    },
    TaskNotification {
        task_id: TaskId,
        kind: String,
        summary: String,
    },
    ExternalTrace {
        source_kind: String,
        source_id: String,
        summary: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiApprovalRequest {
    pub request_id: String,
    pub status: String,
    pub action: String,
    pub resource_kind: String,
    pub resource: String,
    pub reason: String,
    pub preview: UiApprovalPreviewSummary,
    pub max_duration_seconds: Option<u64>,
    pub expires_at: Option<Timestamp>,
}

impl UiApprovalRequest {
    #[must_use]
    pub fn from_pending_approval(approval: &PendingApproval) -> Self {
        Self {
            request_id: approval.id.to_string(),
            status: "pending".to_string(),
            action: capability_action_name(approval.lease_scope.action).to_string(),
            resource_kind: resource_kind_name(approval.lease_scope.resource.kind).to_string(),
            resource: approval.lease_scope.resource.pattern.clone(),
            reason: approval.permission_request.reason.clone(),
            preview: UiApprovalPreviewSummary::from_pending_approval(approval),
            max_duration_seconds: approval.lease_scope.max_duration_seconds,
            expires_at: approval.expires_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiApprovalPreviewSummary {
    pub kind: String,
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub truncated: bool,
    pub redacted: bool,
}

impl UiApprovalPreviewSummary {
    #[must_use]
    pub fn from_pending_approval(approval: &PendingApproval) -> Self {
        Self {
            kind: approval.preview.kind.clone(),
            resource: approval.preview.resource.clone(),
            summary: approval.preview.summary.clone(),
            content_hash: approval.preview.content_hash.clone(),
            truncated: approval.preview.truncated,
            redacted: approval.preview.redacted,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDiagnosticLevel {
    Warning,
    Error,
}

#[must_use]
pub fn normalize_runtime_event(event: &RuntimeEvent) -> UiEventEnvelope {
    UiEventEnvelope {
        schema_version: event.schema_version,
        event_id: event.id,
        seq: event.seq,
        occurred_at: event.occurred_at,
        conversation_id: event.conversation_id,
        turn_id: event.turn_id,
        task_id: event.task_id,
        correlation_id: event.correlation_id,
        payload: normalize_payload(&event.payload),
    }
}

fn normalize_payload(payload: &RuntimeEventPayload) -> UiEventPayload {
    match payload {
        RuntimeEventPayload::WorkspaceOpened(opened) => UiEventPayload::Workspace {
            root_path: opened.root_path.clone(),
            trust_level: trust_level_name(&opened.trust_level).to_string(),
        },
        RuntimeEventPayload::SessionStarted(started) => UiEventPayload::Session {
            client_kind: client_kind_name(&started.client_kind).to_string(),
            client_version: started.client_version.clone(),
        },
        RuntimeEventPayload::UserInputReceived(input) => UiEventPayload::UserMessage {
            message: input.message.clone(),
        },
        RuntimeEventPayload::TurnStateChanged(changed) => UiEventPayload::TurnState {
            state: changed.to,
            reason: changed.reason.clone(),
        },
        RuntimeEventPayload::ModelOutputDelta(delta) => UiEventPayload::ModelDelta {
            delta: delta.delta.clone(),
        },
        RuntimeEventPayload::FinalResponseProduced(response) => UiEventPayload::FinalResponse {
            message: response.message.clone(),
        },
        RuntimeEventPayload::ToolCallProposed(call) => UiEventPayload::ToolCall {
            call_id: call.call_id.clone(),
            tool_name: call.tool_name.clone(),
        },
        RuntimeEventPayload::ToolExecutionCompleted(completed) => UiEventPayload::ToolResult {
            call_id: completed.call_id.clone(),
            ok: true,
            summary: "completed".to_string(),
        },
        RuntimeEventPayload::ToolExecutionFailed(failed) => UiEventPayload::ToolResult {
            call_id: failed.call_id.clone(),
            ok: false,
            summary: failed.message.clone(),
        },
        RuntimeEventPayload::PermissionDecisionRecorded(decision) => UiEventPayload::Permission {
            request_id: decision.request_id.clone(),
            decision: decision.decision.clone(),
            reason: decision.reason.clone(),
        },
        RuntimeEventPayload::ApprovalRequested(requested) => {
            UiEventPayload::ApprovalRequested(Box::new(UiApprovalRequest {
                request_id: requested.request_id.to_string(),
                status: requested.status.clone(),
                action: requested.action.clone(),
                resource_kind: requested.resource_kind.clone(),
                resource: requested.resource.clone(),
                reason: requested.reason.clone(),
                preview: UiApprovalPreviewSummary {
                    kind: requested.preview.kind.clone(),
                    resource: requested.preview.resource.clone(),
                    summary: requested.preview.summary.clone(),
                    content_hash: requested.preview.content_hash.clone(),
                    truncated: requested.preview.truncated,
                    redacted: requested.preview.redacted,
                },
                max_duration_seconds: requested.max_duration_seconds,
                expires_at: requested.expires_at,
            }))
        }
        RuntimeEventPayload::ApprovalResolved(resolved) => UiEventPayload::ApprovalResolved {
            request_id: resolved.request_id.to_string(),
            status: resolved.status.clone(),
            decision: resolved.decision.clone(),
            reason: resolved.reason.clone(),
        },
        RuntimeEventPayload::ApprovalExpired(expired) => UiEventPayload::ApprovalResolved {
            request_id: expired.request_id.to_string(),
            status: "expired".to_string(),
            decision: None,
            reason: Some(expired.reason.clone()),
        },
        RuntimeEventPayload::ApprovalCancelled(cancelled) => UiEventPayload::ApprovalResolved {
            request_id: cancelled.request_id.to_string(),
            status: "cancelled".to_string(),
            decision: None,
            reason: Some(cancelled.reason.clone()),
        },
        RuntimeEventPayload::ApprovalConsumed(consumed) => UiEventPayload::ApprovalResolved {
            request_id: consumed.request_id.to_string(),
            status: "consumed".to_string(),
            decision: Some("allow".to_string()),
            reason: Some(format!(
                "approval consumed by {} {}",
                consumed.tool_name, consumed.call_id
            )),
        },
        RuntimeEventPayload::CheckpointRecorded(checkpoint) => UiEventPayload::Checkpoint {
            checkpoint_id: checkpoint.checkpoint_id,
            label: checkpoint.label.clone(),
        },
        RuntimeEventPayload::RuntimeWarning(warning) => UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Warning,
            code: warning.code.clone(),
            message: warning.message.clone(),
            recoverable: true,
        },
        RuntimeEventPayload::RuntimeError(error) => UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Error,
            code: error.code.clone(),
            message: error.message.clone(),
            recoverable: error.recoverable,
        },
        RuntimeEventPayload::TurnStarted(started) => UiEventPayload::Progress {
            message: started.input_summary.clone(),
        },
        RuntimeEventPayload::ContextAssembled(context) => UiEventPayload::Progress {
            message: format!(
                "context assembled: {} messages, {} estimated tokens",
                context.message_count, context.estimated_tokens
            ),
        },
        RuntimeEventPayload::PromptSnapshotRecorded(snapshot) => {
            UiEventPayload::PromptSummary(prompt_summary_from_snapshot(snapshot))
        }
        RuntimeEventPayload::ModelStreamStarted(started) => UiEventPayload::Progress {
            message: format!(
                "model stream started: {}/{}",
                started.provider, started.model
            ),
        },
        RuntimeEventPayload::ModelStreamCompleted(completed) => UiEventPayload::Progress {
            message: format!("model stream completed: {:?}", completed.finish_reason),
        },
        RuntimeEventPayload::TurnCompleted(completed) => UiEventPayload::Progress {
            message: completed.summary.clone(),
        },
        RuntimeEventPayload::TurnFailed(failed) => UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Error,
            code: failed.error.code.clone(),
            message: failed.error.message.clone(),
            recoverable: failed.error.recoverable,
        },
        RuntimeEventPayload::TaskStateChanged(changed) => UiEventPayload::Progress {
            message: format!("task state changed: {:?} -> {:?}", changed.from, changed.to),
        },
        RuntimeEventPayload::TaskNotificationRecorded(recorded) => {
            UiEventPayload::TaskNotification {
                task_id: recorded.task_id,
                kind: recorded.kind.clone(),
                summary: recorded.summary.clone(),
            }
        }
        RuntimeEventPayload::ToolExecutionStarted(started) => UiEventPayload::Progress {
            message: format!(
                "tool execution started: {} {}",
                started.tool_name, started.call_id
            ),
        },
        RuntimeEventPayload::ToolObservationRecorded(observed) => UiEventPayload::Progress {
            message: format!("tool observation recorded: {}", observed.call_id),
        },
        RuntimeEventPayload::ReplayStarted(replay) => UiEventPayload::Progress {
            message: format!("replay started: {}", replay.source_turn_id),
        },
        RuntimeEventPayload::ReplayCompleted(replay) => UiEventPayload::Progress {
            message: format!("replay completed: {} events", replay.event_count),
        },
        RuntimeEventPayload::McpServerHealthRecorded(recorded) => UiEventPayload::ExternalTrace {
            source_kind: "mcp_server".to_string(),
            source_id: recorded.server_id.clone(),
            summary: format!(
                "MCP server health {}: {}",
                recorded.status, recorded.message
            ),
        },
        RuntimeEventPayload::McpToolsDiscovered(discovered) => UiEventPayload::ExternalTrace {
            source_kind: "mcp_server".to_string(),
            source_id: discovered.server_id.clone(),
            summary: format!(
                "discovered {} MCP tools: {}",
                discovered.tool_count,
                discovered.local_tool_names.join(", ")
            ),
        },
        RuntimeEventPayload::McpPolicyMappingRecorded(mapping) => UiEventPayload::ExternalTrace {
            source_kind: "mcp_server".to_string(),
            source_id: mapping.server_id.clone(),
            summary: format!(
                "mapped MCP tool {} -> {} action={} scope={}:{}",
                mapping.remote_tool_name,
                mapping.local_tool_name,
                mapping.action,
                mapping.resource_kind,
                mapping.resource_pattern
            ),
        },
        RuntimeEventPayload::SkillDiscoveredRecorded(discovered) => UiEventPayload::ExternalTrace {
            source_kind: "skill".to_string(),
            source_id: discovered.skill_id.clone(),
            summary: format!(
                "discovered skill {} from {}",
                discovered.name, discovered.source_path
            ),
        },
        RuntimeEventPayload::SkillPromptContributionRecorded(contribution) => {
            UiEventPayload::ExternalTrace {
                source_kind: "skill".to_string(),
                source_id: contribution.skill_id.clone(),
                summary: format!(
                    "skill prompt layer {} priority={} tokens={} source={}",
                    contribution.layer_id,
                    contribution.priority,
                    contribution.token_estimate,
                    contribution.source_path
                ),
            }
        }
        RuntimeEventPayload::PluginCapabilityValidated(validated) => {
            UiEventPayload::ExternalTrace {
                source_kind: "plugin".to_string(),
                source_id: validated.plugin_name.clone(),
                summary: format!(
                    "validated {} plugin capabilities: tools={} hooks={}",
                    validated.capability_count, validated.tool_count, validated.hook_count
                ),
            }
        }
        RuntimeEventPayload::ConversationStarted(started) => UiEventPayload::Progress {
            message: started.title.clone(),
        },
    }
}

fn prompt_summary_from_snapshot(snapshot: &agent_types::PromptSnapshotRecorded) -> UiPromptSummary {
    let profile_id = non_empty_or(snapshot.profile_id.clone(), "unknown");
    let bundle_hash = non_empty_or(snapshot.bundle_hash.clone(), "unknown");
    let layers = snapshot
        .layer_hashes
        .iter()
        .map(|layer| {
            UiPromptLayerSummary::new(
                non_empty_or(layer.layer_id.clone(), "unknown"),
                "unknown",
                "snapshot",
                0,
                0,
                "summary_only",
                "redacted",
                None,
                Some(layer.source_hash.clone()),
                Some(layer.content_hash.clone()),
            )
            .unwrap_or_else(|_| UiPromptLayerSummary {
                layer_id: "unknown".to_string(),
                kind: "unknown".to_string(),
                source_label: "snapshot".to_string(),
                priority: 0,
                token_estimate: 0,
                visibility: "summary_only".to_string(),
                sensitivity: "redacted".to_string(),
                content_preview: None,
                source_hash: None,
                content_hash: None,
            })
        })
        .collect();
    let mut diagnostics = vec![
        UiPromptDiagnosticSummary::new(
            "info",
            "prompt.snapshot.partial_metadata",
            "prompt snapshot contains layer hashes but not full layer metadata",
            None,
        )
        .expect("static diagnostic is valid"),
    ];
    if snapshot.diagnostic_count > 0 {
        diagnostics.push(
            UiPromptDiagnosticSummary::new(
                "warning",
                "prompt.snapshot.diagnostics_recorded",
                format!(
                    "{} prompt diagnostics were recorded in the runtime snapshot",
                    snapshot.diagnostic_count
                ),
                None,
            )
            .expect("static diagnostic is valid"),
        );
    }

    UiPromptSummary::new(
        profile_id.clone(),
        snapshot.profile_version.max(1),
        profile_id,
        bundle_hash,
        snapshot.estimated_tokens,
        snapshot.message_count,
        layers,
        diagnostics,
    )
    .expect("snapshot prompt summary uses fallback values")
}

fn non_empty_or(value: String, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn trust_level_name(level: &WorkspaceTrustLevel) -> &'static str {
    match level {
        WorkspaceTrustLevel::Trusted => "trusted",
        WorkspaceTrustLevel::Untrusted => "untrusted",
    }
}

fn client_kind_name(kind: &ClientKind) -> &'static str {
    match kind {
        ClientKind::Cli => "cli",
        ClientKind::Tui => "tui",
        ClientKind::Desktop => "desktop",
        ClientKind::Api => "api",
        ClientKind::Test => "test",
    }
}

fn capability_action_name(action: CapabilityAction) -> &'static str {
    match action {
        CapabilityAction::Read => "read",
        CapabilityAction::Search => "search",
        CapabilityAction::Execute => "execute",
        CapabilityAction::Network => "network",
    }
}

fn resource_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::File => "file",
        ResourceKind::Workspace => "workspace",
        ResourceKind::Command => "command",
        ResourceKind::Network => "network",
        ResourceKind::Tool => "tool",
    }
}
