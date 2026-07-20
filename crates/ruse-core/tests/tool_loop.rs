use std::fs;

use ruse_core::{RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopStatus};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_sandbox::WorkspaceSandbox;
use ruse_tool::ToolRegistry;
use ruse_tool::builtins::ReadFileTool;
use ruse_types::{
    ConversationId, CorrelationId, RuntimeEventPayload, TurnId, TurnState, WorkspaceId,
};
use serde_json::json;
use tempfile::tempdir;

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn registry_with_read_file() -> (tempfile::TempDir, ToolRegistry) {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(root.join("README.md"), "hello from tool\n").expect("readme");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    let mut registry = ToolRegistry::default();
    registry
        .register(ReadFileTool::new(sandbox))
        .expect("register read file");
    (temp, registry)
}

fn call_request() -> ToolCallRequest {
    ToolCallRequest {
        call_id: "call-1".to_string(),
        tool_name: "workspace.read_file".to_string(),
        input_json: json!({ "path": "README.md" }),
    }
}

fn policy(decision: PolicyDecision) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "workspace.read_file").expect("scope"),
        decision,
    )
}

#[test]
fn tool_loop_allow_executes_tool_and_records_audit_order() {
    let (_temp, registry) = registry_with_read_file();
    let outcome = ToolLoop
        .run(
            runtime_ids(),
            call_request(),
            &registry,
            &policy(PolicyDecision::Allow {
                reason: "read-only tool".to_string(),
            }),
        )
        .expect("tool loop");

    assert_eq!(outcome.status, ToolLoopStatus::Completed);
    assert_eq!(outcome.observation.as_deref(), Some("hello from tool\n"));
    let names: Vec<&str> = outcome
        .events
        .iter()
        .map(|event| match &event.payload {
            RuntimeEventPayload::ToolCallProposed(_) => "proposed",
            RuntimeEventPayload::PermissionDecisionRecorded(_) => "decision",
            RuntimeEventPayload::ToolExecutionStarted(_) => "started",
            RuntimeEventPayload::ToolExecutionCompleted(_) => "completed",
            RuntimeEventPayload::ToolObservationRecorded(_) => "observed",
            _ => "unexpected",
        })
        .collect();
    assert_eq!(
        names,
        vec!["proposed", "decision", "started", "completed", "observed"]
    );
    let started = outcome
        .events
        .iter()
        .find_map(|event| match &event.payload {
            RuntimeEventPayload::ToolExecutionStarted(started) => Some(started),
            _ => None,
        })
        .expect("tool started event");
    assert_eq!(started.policy_decision.as_deref(), Some("allow"));
    assert_eq!(started.action.as_deref(), Some("read"));
    assert_eq!(started.resource_kind.as_deref(), Some("tool"));
    assert_eq!(started.resource.as_deref(), Some("workspace.read_file"));
    assert_eq!(started.sandbox_profile.as_deref(), Some("workspace_fs"));
    let completed = outcome
        .events
        .iter()
        .find_map(|event| match &event.payload {
            RuntimeEventPayload::ToolExecutionCompleted(completed) => Some(completed),
            _ => None,
        })
        .expect("tool completed event");
    assert!(completed.elapsed_ms.is_some());
}

#[test]
fn tool_loop_deny_does_not_execute_tool() {
    let (_temp, registry) = registry_with_read_file();
    let outcome = ToolLoop
        .run(
            runtime_ids(),
            call_request(),
            &registry,
            &policy(PolicyDecision::Deny {
                reason: "denied by test".to_string(),
            }),
        )
        .expect("tool loop");

    assert_eq!(outcome.status, ToolLoopStatus::Denied);
    assert!(outcome.observation.is_none());
    assert!(
        !outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
    );
}

#[test]
fn tool_loop_ask_waits_for_approval_without_execution() {
    let (_temp, registry) = registry_with_read_file();
    let outcome = ToolLoop
        .run(
            runtime_ids(),
            call_request(),
            &registry,
            &policy(PolicyDecision::Ask {
                reason: "needs user approval".to_string(),
            }),
        )
        .expect("tool loop");

    assert_eq!(outcome.status, ToolLoopStatus::WaitingApproval);
    assert_eq!(outcome.turn_state, TurnState::WaitingApproval);
    assert!(outcome.observation.is_none());
    assert!(
        !outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
    );
}
