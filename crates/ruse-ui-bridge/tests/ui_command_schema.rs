use ruse_types::{ClientKind, ConversationId, CorrelationId, SchemaVersion, TurnId, WorkspaceId};
use ruse_ui_bridge::{
    UiBridgeError, UiClientInfo, UiCommand, UiCommandEnvelope, UiPermissionDecision,
    validate_ui_command,
};
use serde_json::json;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn command_id() -> CorrelationId {
    id("00000000-0000-0000-0000-000000000100")
}

fn client() -> UiClientInfo {
    UiClientInfo {
        kind: ClientKind::Desktop,
        version: "0.1.0".to_string(),
    }
}

fn envelope(command: UiCommand) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: command_id(),
        client: client(),
        command,
    }
}

#[test]
fn ui_command_schema_deserializes_start_turn() {
    let value = json!({
        "schema_version": 1,
        "command_id": "00000000-0000-0000-0000-000000000100",
        "client": {
            "kind": "desktop",
            "version": "0.1.0"
        },
        "command": {
            "type": "start_turn",
            "data": {
                "workspace_id": "00000000-0000-0000-0000-000000000001",
                "conversation_id": "00000000-0000-0000-0000-000000000002",
                "prompt": "Summarize the project"
            }
        }
    });

    let command: UiCommandEnvelope = serde_json::from_value(value).expect("deserialize command");

    assert_eq!(
        command.command,
        UiCommand::StartTurn {
            workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
            conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
            prompt: "Summarize the project".to_string(),
        }
    );
    validate_ui_command(&command).expect("valid start turn command");
}

#[test]
fn ui_command_schema_serializes_permission_response() {
    let command = envelope(UiCommand::PermissionResponse {
        request_id: "perm-1".to_string(),
        decision: UiPermissionDecision::Allow,
        reason: "approved by user".to_string(),
    });

    let value = serde_json::to_value(command).expect("serialize command");

    assert_eq!(
        value,
        json!({
            "schema_version": 1,
            "command_id": "00000000-0000-0000-0000-000000000100",
            "client": {
                "kind": "desktop",
                "version": "0.1.0"
            },
            "command": {
                "type": "permission_response",
                "data": {
                    "request_id": "perm-1",
                    "decision": "allow",
                    "reason": "approved by user"
                }
            }
        })
    );
}

#[test]
fn ui_command_schema_rejects_unknown_command_type() {
    let value = json!({
        "schema_version": 1,
        "command_id": "00000000-0000-0000-0000-000000000100",
        "client": {
            "kind": "desktop",
            "version": "0.1.0"
        },
        "command": {
            "type": "launch_missiles",
            "data": {}
        }
    });

    let error = serde_json::from_value::<UiCommandEnvelope>(value)
        .expect_err("unknown command type rejected");

    assert!(error.to_string().contains("unknown variant"));
}

#[test]
fn ui_command_schema_validates_empty_prompt() {
    let command = envelope(UiCommand::StartTurn {
        workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        prompt: " ".to_string(),
    });

    let error = validate_ui_command(&command).expect_err("empty prompt rejected");

    assert!(matches!(
        error,
        UiBridgeError::Validation(message) if message.contains("prompt")
    ));
}

#[test]
fn ui_command_schema_validates_empty_permission_request() {
    let command = envelope(UiCommand::PermissionResponse {
        request_id: " ".to_string(),
        decision: UiPermissionDecision::Deny,
        reason: " ".to_string(),
    });

    let error = validate_ui_command(&command).expect_err("empty request rejected");

    assert!(matches!(
        error,
        UiBridgeError::Validation(message) if message.contains("request_id")
    ));
}

#[test]
fn ui_command_schema_validates_empty_cancel_reason() {
    let command = envelope(UiCommand::CancelTurn {
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        turn_id: id::<TurnId>("00000000-0000-0000-0000-000000000003"),
        reason: " ".to_string(),
    });

    let error = validate_ui_command(&command).expect_err("empty cancel reason rejected");

    assert!(matches!(
        error,
        UiBridgeError::Validation(message) if message.contains("reason")
    ));
}
