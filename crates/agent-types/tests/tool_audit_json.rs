use agent_types::{
    PermissionDecisionRecorded, RuntimeEventPayload, ToolCallProposed, ToolExecutionCompleted,
    ToolExecutionFailed, ToolExecutionStarted, ToolObservationRecorded,
};
use serde_json::json;

#[test]
fn tool_call_proposed_has_stable_json_shape() {
    let payload = RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
        call_id: "call-1".to_string(),
        tool_name: "workspace.read_file".to_string(),
        input_json: json!({ "path": "README.md" }),
    });

    assert_eq!(
        serde_json::to_value(payload).expect("serialize payload"),
        json!({
            "type": "tool_call_proposed",
            "data": {
                "call_id": "call-1",
                "tool_name": "workspace.read_file",
                "input_json": { "path": "README.md" }
            }
        })
    );
}

#[test]
fn permission_decision_and_tool_result_have_stable_json_shape() {
    let decision = RuntimeEventPayload::PermissionDecisionRecorded(PermissionDecisionRecorded {
        request_id: "request-1".to_string(),
        decision: "allow".to_string(),
        reason: "read-only path".to_string(),
        action: None,
        resource_kind: None,
        resource: None,
    });
    let completed = RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
        call_id: "call-1".to_string(),
        output_json: json!({ "content": "hello" }),
        elapsed_ms: None,
    });

    assert_eq!(
        serde_json::to_value(decision).expect("serialize decision"),
        json!({
            "type": "permission_decision_recorded",
            "data": {
                "request_id": "request-1",
                "decision": "allow",
                "reason": "read-only path"
            }
        })
    );
    assert_eq!(
        serde_json::to_value(completed).expect("serialize completion"),
        json!({
            "type": "tool_execution_completed",
            "data": {
                "call_id": "call-1",
                "output_json": { "content": "hello" }
            }
        })
    );
}

#[test]
fn all_tool_audit_payloads_round_trip() {
    let payloads = vec![
        RuntimeEventPayload::ToolExecutionStarted(ToolExecutionStarted {
            call_id: "call-1".to_string(),
            tool_name: "workspace.read_file".to_string(),
            policy_decision: None,
            policy_reason: None,
            action: None,
            resource_kind: None,
            resource: None,
            sandbox_profile: None,
        }),
        RuntimeEventPayload::ToolExecutionFailed(ToolExecutionFailed {
            call_id: "call-2".to_string(),
            error_code: "tool_failed".to_string(),
            message: "boom".to_string(),
            recoverable: true,
            elapsed_ms: None,
        }),
        RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
            call_id: "call-1".to_string(),
            observation: "read README.md".to_string(),
        }),
    ];

    for payload in payloads {
        let json = serde_json::to_string(&payload).expect("serialize payload");
        let round_tripped: RuntimeEventPayload =
            serde_json::from_str(&json).expect("deserialize payload");
        assert_eq!(round_tripped, payload);
    }
}
