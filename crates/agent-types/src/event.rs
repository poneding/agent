use serde::{Deserialize, Serialize};

use crate::actor::ActorRef;
use crate::error::AgentError;
use crate::ids::{
    AgentId, CheckpointId, ConversationId, CorrelationId, EventId, EventSeq, TaskId, TeamId,
    TurnId, WorkspaceId,
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
    ModelStreamStarted(ModelStreamStarted),
    ModelOutputDelta(ModelOutputDelta),
    ModelStreamCompleted(ModelStreamCompleted),
    FinalResponseProduced(FinalResponseProduced),
    TurnCompleted(TurnCompleted),
    TurnFailed(TurnFailed),
    TaskStateChanged(TaskStateChanged),
    ToolCallProposed(crate::tool::ToolCallProposed),
    PermissionDecisionRecorded(crate::tool::PermissionDecisionRecorded),
    ToolExecutionStarted(crate::tool::ToolExecutionStarted),
    ToolExecutionCompleted(crate::tool::ToolExecutionCompleted),
    ToolExecutionFailed(crate::tool::ToolExecutionFailed),
    ToolObservationRecorded(crate::tool::ToolObservationRecorded),
    CheckpointRecorded(CheckpointRecorded),
    ReplayStarted(ReplayStarted),
    ReplayCompleted(ReplayCompleted),
    McpServerHealthRecorded(crate::external::McpServerHealthRecorded),
    McpToolsDiscovered(crate::external::McpToolsDiscovered),
    McpPolicyMappingRecorded(crate::external::McpPolicyMappingRecorded),
    SkillDiscoveredRecorded(crate::external::SkillDiscoveredRecorded),
    SkillPromptContributionRecorded(crate::external::SkillPromptContributionRecorded),
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
pub struct ModelStreamStarted {
    pub provider: String,
    pub model: String,
    pub request_id: String,
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
