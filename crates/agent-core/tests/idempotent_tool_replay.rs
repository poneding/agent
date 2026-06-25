use std::fs;

use agent_core::{IdempotentToolExecutor, RuntimeIds, ToolCallRequest, ToolLoopStatus};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_sandbox::WorkspaceSandbox;
use agent_storage::{IdempotencyKey, IdempotencyStore, InMemoryEventStore};
use agent_tool::ToolRegistry;
use agent_tool::builtins::FileEditTool;
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
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

fn edit_fixture() -> (tempfile::TempDir, ToolRegistry) {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(root.join("README.md"), "old text\n").expect("readme");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    let mut registry = ToolRegistry::default();
    registry
        .register(FileEditTool::new(sandbox))
        .expect("register edit file");
    (temp, registry)
}

fn edit_call(replacement: &str) -> ToolCallRequest {
    edit_call_with_expected("old text\n", replacement)
}

fn edit_call_with_expected(expected: &str, replacement: &str) -> ToolCallRequest {
    ToolCallRequest {
        call_id: "call-edit".to_string(),
        tool_name: "workspace.edit_file".to_string(),
        input_json: json!({
            "path": "README.md",
            "expected": expected,
            "replacement": replacement,
            "apply": true
        }),
    }
}

fn edit_policy() -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "workspace.edit_file").expect("scope"),
        PolicyDecision::Allow {
            reason: "allow edit fixture".to_string(),
        },
    )
}

#[test]
fn idempotent_tool_replay_runs_first_execution_and_records_effect() {
    let (temp, registry) = edit_fixture();
    let mut effects = InMemoryEventStore::default();

    let outcome = IdempotentToolExecutor::default()
        .run(
            runtime_ids(),
            IdempotencyKey::new("turn-1:call-edit").expect("key"),
            edit_call("new text\n"),
            &registry,
            &edit_policy(),
            &mut effects,
        )
        .expect("idempotent execution");

    assert_eq!(outcome.tool.status, ToolLoopStatus::Completed);
    assert!(!outcome.replayed);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "new text\n"
    );
    assert!(
        effects
            .get_effect(&IdempotencyKey::new("turn-1:call-edit").expect("key"))
            .expect("get effect")
            .is_some()
    );
}

#[test]
fn idempotent_tool_replay_returns_recorded_output_without_second_execution() {
    let (temp, registry) = edit_fixture();
    let key = IdempotencyKey::new("turn-1:call-edit").expect("key");
    let mut effects = InMemoryEventStore::default();
    let executor = IdempotentToolExecutor::default();

    let first = executor
        .run(
            runtime_ids(),
            key.clone(),
            edit_call("new text\n"),
            &registry,
            &edit_policy(),
            &mut effects,
        )
        .expect("first execution");
    fs::write(temp.path().join("workspace/README.md"), "old text\n").expect("reset fixture");

    let second = executor
        .run(
            runtime_ids(),
            key,
            edit_call("new text\n"),
            &registry,
            &edit_policy(),
            &mut effects,
        )
        .expect("second execution");

    assert_eq!(first.tool.status, ToolLoopStatus::Completed);
    assert_eq!(second.tool.status, ToolLoopStatus::Completed);
    assert!(second.replayed);
    assert_eq!(second.tool.events.len(), 1);
    assert!(matches!(
        second.tool.events[0].payload,
        RuntimeEventPayload::ToolObservationRecorded(_)
    ));
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n",
        "second execution should not mutate the file"
    );
}

#[test]
fn idempotent_tool_replay_does_not_record_failed_tool_calls() {
    let (_temp, registry) = edit_fixture();
    let key = IdempotencyKey::new("turn-1:call-edit").expect("key");
    let mut effects = InMemoryEventStore::default();

    let outcome = IdempotentToolExecutor::default()
        .run(
            runtime_ids(),
            key.clone(),
            edit_call_with_expected("not current text\n", "new text\n"),
            &registry,
            &edit_policy(),
            &mut effects,
        )
        .expect("failed tool execution is still an outcome");

    assert_eq!(outcome.tool.status, ToolLoopStatus::Failed);
    assert!(!outcome.replayed);
    assert!(effects.get_effect(&key).expect("get effect").is_none());
}
