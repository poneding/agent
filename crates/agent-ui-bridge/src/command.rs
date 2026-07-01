use agent_policy::ApprovalResponse;
use agent_scheduler::TaskCommand;
use agent_types::{
    ApprovalRequestId, ClientKind, ConversationId, CorrelationId, PlanTodoItemId, PlanTodoSnapshot,
    SchemaVersion, Timestamp, TodoStatus, TurnId, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use crate::UiBridgeError;

const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiCommandEnvelope {
    pub schema_version: SchemaVersion,
    pub command_id: CorrelationId,
    pub client: UiClientInfo,
    pub command: UiCommand,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiClientInfo {
    pub kind: ClientKind,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiCommand {
    StartTurn {
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
        prompt: String,
    },
    CancelTurn {
        conversation_id: ConversationId,
        turn_id: TurnId,
        reason: String,
    },
    PermissionResponse {
        request_id: String,
        decision: UiPermissionDecision,
        reason: String,
    },
    ResolveApproval {
        request_id: ApprovalRequestId,
        decision: UiApprovalDecision,
        reason: String,
        lease_duration_seconds: Option<u64>,
    },
    TraceRequest {
        conversation_id: ConversationId,
        include_tools: bool,
        include_policy: bool,
        include_checkpoints: bool,
        include_final: bool,
    },
    Task(TaskCommand),
    PlanTodo(UiPlanTodoCommand),
    SwitchSession {
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
    },
    Ping,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiPlanTodoCommand {
    Show {
        conversation_id: ConversationId,
    },
    List {
        conversation_id: ConversationId,
    },
    UpdateStatus {
        conversation_id: ConversationId,
        item_id: PlanTodoItemId,
        status: TodoStatus,
        expected_revision: u64,
        reason: String,
    },
    ReplaceSnapshot {
        snapshot: PlanTodoSnapshot,
        expected_revision: u64,
        reason: String,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPermissionDecision {
    Allow,
    Deny,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiApprovalDecision {
    Allow,
    Deny,
}

impl UiCommand {
    pub fn to_approval_response(
        &self,
        resolved_at: Timestamp,
    ) -> Result<ApprovalResponse, UiBridgeError> {
        match self {
            Self::ResolveApproval {
                request_id,
                decision,
                reason,
                lease_duration_seconds,
            } => match decision {
                UiApprovalDecision::Allow => ApprovalResponse::allow(
                    *request_id,
                    reason.clone(),
                    *lease_duration_seconds,
                    resolved_at,
                )
                .map_err(|error| UiBridgeError::Validation(error.to_string())),
                UiApprovalDecision::Deny => {
                    ApprovalResponse::deny(*request_id, reason.clone(), resolved_at)
                        .map_err(|error| UiBridgeError::Validation(error.to_string()))
                }
            },
            _ => Err(UiBridgeError::Validation(
                "command is not resolve_approval".to_string(),
            )),
        }
    }
}

pub fn validate_ui_command(envelope: &UiCommandEnvelope) -> Result<(), UiBridgeError> {
    if envelope.client.version.trim().is_empty() {
        return Err(UiBridgeError::Validation(
            "client version cannot be empty".to_string(),
        ));
    }

    match &envelope.command {
        UiCommand::StartTurn { prompt, .. } if prompt.trim().is_empty() => Err(
            UiBridgeError::Validation("start_turn prompt cannot be empty".to_string()),
        ),
        UiCommand::CancelTurn { reason, .. } if reason.trim().is_empty() => Err(
            UiBridgeError::Validation("cancel_turn reason cannot be empty".to_string()),
        ),
        UiCommand::PermissionResponse { request_id, .. } if request_id.trim().is_empty() => Err(
            UiBridgeError::Validation("permission_response request_id cannot be empty".to_string()),
        ),
        UiCommand::PermissionResponse { reason, .. } if reason.trim().is_empty() => Err(
            UiBridgeError::Validation("permission_response reason cannot be empty".to_string()),
        ),
        UiCommand::ResolveApproval { reason, .. } if reason.trim().is_empty() => Err(
            UiBridgeError::Validation("resolve_approval reason cannot be empty".to_string()),
        ),
        UiCommand::PlanTodo(command) => validate_plan_todo_command(command),
        UiCommand::SwitchSession { workspace_id, .. } if is_nil_workspace_id(*workspace_id) => Err(
            UiBridgeError::Validation("switch_session workspace_id cannot be nil".to_string()),
        ),
        UiCommand::SwitchSession {
            conversation_id, ..
        } if is_nil_conversation_id(*conversation_id) => Err(UiBridgeError::Validation(
            "switch_session conversation_id cannot be nil".to_string(),
        )),
        _ => Ok(()),
    }
}

fn validate_plan_todo_command(command: &UiPlanTodoCommand) -> Result<(), UiBridgeError> {
    match command {
        UiPlanTodoCommand::Show { conversation_id }
        | UiPlanTodoCommand::List { conversation_id }
        | UiPlanTodoCommand::UpdateStatus {
            conversation_id, ..
        } if is_nil_conversation_id(*conversation_id) => Err(UiBridgeError::Validation(
            "plan_todo conversation_id cannot be nil".to_string(),
        )),
        UiPlanTodoCommand::UpdateStatus { reason, .. }
        | UiPlanTodoCommand::ReplaceSnapshot { reason, .. }
            if reason.trim().is_empty() =>
        {
            Err(UiBridgeError::Validation(
                "plan_todo reason cannot be empty".to_string(),
            ))
        }
        UiPlanTodoCommand::ReplaceSnapshot { snapshot, .. }
            if is_nil_conversation_id(snapshot.conversation_id) =>
        {
            Err(UiBridgeError::Validation(
                "plan_todo snapshot conversation_id cannot be nil".to_string(),
            ))
        }
        _ => Ok(()),
    }
}

fn is_nil_workspace_id(workspace_id: WorkspaceId) -> bool {
    workspace_id.to_string() == NIL_UUID
}

fn is_nil_conversation_id(conversation_id: ConversationId) -> bool {
    conversation_id.to_string() == NIL_UUID
}
