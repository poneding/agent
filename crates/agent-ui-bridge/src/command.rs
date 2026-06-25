use agent_types::{ClientKind, ConversationId, CorrelationId, SchemaVersion, TurnId, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::UiBridgeError;

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
    TraceRequest {
        conversation_id: ConversationId,
        include_tools: bool,
        include_policy: bool,
        include_checkpoints: bool,
        include_final: bool,
    },
    Ping,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPermissionDecision {
    Allow,
    Deny,
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
        _ => Ok(()),
    }
}
