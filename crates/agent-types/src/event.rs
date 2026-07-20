use serde::{Deserialize, Serialize};

use crate::actor::ActorRef;
use crate::error::AgentError;
use crate::ids::{
    AgentId, ApprovalRequestId, CheckpointId, ConversationId, CorrelationId, EventId, EventSeq,
    TaskId, TeamId, TurnId, WorkspaceId,
};
use crate::schema::SchemaVersion;
use crate::state::{TaskState, TurnState};
use crate::timestamp::Timestamp;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceTrustLevel {
    Trusted,
    Untrusted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Cli,
    Tui,
    Desktop,
    Api,
    Test,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFinishReason {
    Stop,
    Length,
    Cancelled,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub schema_version: SchemaVersion,
    pub id: EventId,
    pub seq: Option<EventSeq>,
    pub occurred_at: Timestamp,
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub task_id: Option<TaskId>,
    pub agent_id: Option<AgentId>,
    pub team_id: Option<TeamId>,
    pub actor: ActorRef,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub payload: RuntimeEventPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum RuntimeEventPayload {
    WorkspaceOpened(WorkspaceOpened),
    SessionStarted(SessionStarted),
    ConversationStarted(ConversationStarted),
    UserInputReceived(UserInputReceived),
    TurnStarted(TurnStarted),
    TurnStateChanged(TurnStateChanged),
    ContextAssembled(ContextAssembled),
    PromptSnapshotRecorded(PromptSnapshotRecorded),
    ModelStreamStarted(ModelStreamStarted),
    ModelReasoningSummaryDelta(ModelReasoningSummaryDelta),
    ModelOutputDelta(ModelOutputDelta),
    ModelStreamCompleted(ModelStreamCompleted),
    FinalResponseProduced(FinalResponseProduced),
    TurnCompleted(TurnCompleted),
    TurnFailed(TurnFailed),
    PlanTodoSnapshotRecorded(crate::plan::PlanTodoSnapshotRecorded),
    PlanTodoUpdated(crate::plan::PlanTodoUpdated),
    TaskStateChanged(TaskStateChanged),
    TaskNotificationRecorded(crate::task::TaskNotificationRecorded),
    ToolCallProposed(crate::tool::ToolCallProposed),
    PermissionDecisionRecorded(crate::tool::PermissionDecisionRecorded),
    ToolExecutionStarted(crate::tool::ToolExecutionStarted),
    ToolExecutionCompleted(crate::tool::ToolExecutionCompleted),
    ToolExecutionFailed(crate::tool::ToolExecutionFailed),
    ToolObservationRecorded(crate::tool::ToolObservationRecorded),
    ApprovalRequested(ApprovalRequested),
    ApprovalResolved(ApprovalResolved),
    ApprovalExpired(ApprovalExpired),
    ApprovalCancelled(ApprovalCancelled),
    ApprovalConsumed(ApprovalConsumed),
    CheckpointRecorded(CheckpointRecorded),
    ReplayStarted(ReplayStarted),
    ReplayCompleted(ReplayCompleted),
    McpServerHealthRecorded(crate::external::McpServerHealthRecorded),
    McpToolsDiscovered(crate::external::McpToolsDiscovered),
    McpPolicyMappingRecorded(crate::external::McpPolicyMappingRecorded),
    SkillDiscoveredRecorded(crate::external::SkillDiscoveredRecorded),
    SkillPromptContributionRecorded(crate::external::SkillPromptContributionRecorded),
    PluginCapabilityValidated(crate::external::PluginCapabilityValidated),
    RuntimeWarning(crate::error::RuntimeWarning),
    RuntimeError(crate::error::RuntimeError),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpened {
    pub root_path: String,
    pub trust_level: WorkspaceTrustLevel,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionStarted {
    pub client_kind: ClientKind,
    pub client_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationStarted {
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserInputReceived {
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnStarted {
    pub input_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnStateChanged {
    pub from: TurnState,
    pub to: TurnState,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextAssembled {
    pub message_count: u32,
    pub estimated_tokens: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptLayerHashRecord {
    pub layer_id: String,
    pub source_hash: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptSnapshotRecorded {
    pub profile_id: String,
    pub profile_version: u32,
    pub bundle_hash: String,
    pub layer_hashes: Vec<PromptLayerHashRecord>,
    pub diagnostic_count: u32,
    pub message_count: u32,
    pub estimated_tokens: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelStreamStarted {
    pub provider: String,
    pub model: String,
    pub request_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelReasoningSummaryDelta {
    pub delta: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelOutputDelta {
    pub delta: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelStreamCompleted {
    pub finish_reason: ModelFinishReason,
    pub output_token_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinalResponseProduced {
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnCompleted {
    pub final_state: TurnState,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnFailed {
    pub state: TurnState,
    pub error: AgentError,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskStateChanged {
    pub task_id: TaskId,
    pub from: TaskState,
    pub to: TaskState,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRecorded {
    pub checkpoint_id: CheckpointId,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequested {
    pub request_id: ApprovalRequestId,
    pub status: String,
    pub action: String,
    pub resource_kind: String,
    pub resource: String,
    pub reason: String,
    pub preview: ApprovalPreviewRecorded,
    pub checkpoint_id: Option<CheckpointId>,
    pub max_duration_seconds: Option<u64>,
    pub expires_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalPreviewRecorded {
    pub kind: String,
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalResolved {
    pub request_id: ApprovalRequestId,
    pub status: String,
    pub decision: Option<String>,
    pub reason: Option<String>,
    pub lease_duration_seconds: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalExpired {
    pub request_id: ApprovalRequestId,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalCancelled {
    pub request_id: ApprovalRequestId,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalConsumed {
    pub request_id: ApprovalRequestId,
    pub call_id: String,
    pub tool_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayStarted {
    pub source_conversation_id: ConversationId,
    pub source_turn_id: TurnId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayCompleted {
    pub source_conversation_id: ConversationId,
    pub source_turn_id: TurnId,
    pub event_count: u64,
}
