use std::fs;

use ruse_core::{
    ApprovalCoordinator, ApprovalRecoveryView, ApprovalResumeError, ApprovalResumeStatus,
    RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopStatus,
};
use ruse_policy::{
    ApprovalResponse, CapabilityAction, PolicyDecision, ResourceKind, ResourceScope,
    StaticPolicyEngine,
};
use ruse_sandbox::WorkspaceSandbox;
use ruse_storage::{EventStore, InMemoryEventStore};
use ruse_tool::ToolRegistry;
use ruse_tool::builtins::FileEditTool;
use ruse_types::{
    ConversationId, CorrelationId, RuntimeEventPayload, Timestamp, TurnId, TurnState, WorkspaceId,
};
use serde_json::json;
use tempfile::TempDir;

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn ts(second: u8) -> Timestamp {
    let value = format!("\"2026-06-27T10:00:{second:02}Z\"");
    serde_json::from_str(&value).expect("valid timestamp")
}

fn edit_registry() -> (TempDir, ToolRegistry) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace dir");
    fs::write(root.join("README.md"), "old text\n").expect("readme");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    let mut registry = ToolRegistry::default();
    registry
        .register(FileEditTool::new(sandbox))
        .expect("register edit tool");
    (temp, registry)
}

fn edit_call() -> ToolCallRequest {
    ToolCallRequest {
        call_id: "call-edit".to_string(),
        tool_name: "workspace.edit_file".to_string(),
        input_json: json!({
            "path": "README.md",
            "expected": "old text\n",
            "replacement": "new text\n",
            "apply": true
        }),
    }
}

fn ask_policy() -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "workspace.edit_file").expect("scope"),
        PolicyDecision::Ask {
            reason: "needs approval".to_string(),
        },
    )
}

fn pending_tool_call(
    ids: RuntimeIds,
    registry: &ToolRegistry,
) -> (
    ToolCallRequest,
    ruse_policy::PendingApproval,
    Vec<ruse_types::RuntimeEvent>,
) {
    let call = edit_call();
    let outcome = ToolLoop
        .run(ids, call.clone(), registry, &ask_policy())
        .expect("tool loop");

    assert_eq!(outcome.status, ToolLoopStatus::WaitingApproval);
    assert_eq!(outcome.turn_state, TurnState::WaitingApproval);
    assert!(
        !outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
    );

    (
        call,
        outcome.pending_approval.expect("pending approval"),
        outcome.events,
    )
}

#[test]
fn approval_resume_approves_pending_tool_and_executes_once() {
    let ids = runtime_ids();
    let (temp, registry) = edit_registry();
    let (call, pending, events) = pending_tool_call(ids, &registry);
    let request_id = pending.id;
    let mut coordinator = ApprovalCoordinator::default();
    let mut effects = InMemoryEventStore::default();

    coordinator
        .register_suspended_tool(ids, 0, call, pending)
        .expect("register suspended approval");

    let resumed = coordinator
        .resume_tool_approval(
            ApprovalResponse::allow(request_id, "approved", Some(60), ts(1)).expect("response"),
            &registry,
            &mut effects,
        )
        .expect("resume approval");

    assert_eq!(resumed.status, ApprovalResumeStatus::Executed);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "new text\n"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ApprovalRequested(_)))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::CheckpointRecorded(_)))
    );
    assert!(
        resumed
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ApprovalResolved(_)))
    );
    assert!(
        resumed
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
    );
    assert!(
        resumed
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ApprovalConsumed(_)))
    );
}

#[test]
fn approval_resume_denies_pending_tool_without_execution() {
    let ids = runtime_ids();
    let (temp, registry) = edit_registry();
    let (call, pending, _events) = pending_tool_call(ids, &registry);
    let request_id = pending.id;
    let mut coordinator = ApprovalCoordinator::default();
    let mut effects = InMemoryEventStore::default();

    coordinator
        .register_suspended_tool(ids, 0, call, pending)
        .expect("register suspended approval");

    let resumed = coordinator
        .resume_tool_approval(
            ApprovalResponse::deny(request_id, "too risky", ts(1)).expect("response"),
            &registry,
            &mut effects,
        )
        .expect("resume denial");

    assert_eq!(resumed.status, ApprovalResumeStatus::Denied);
    assert!(resumed.tool.is_none());
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n"
    );
    assert!(resumed.events.iter().any(|event| matches!(
        &event.payload,
        RuntimeEventPayload::ApprovalResolved(resolved)
            if resolved.decision.as_deref() == Some("deny")
    )));
    assert!(
        !resumed
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
    );
}

#[test]
fn approval_resume_rejects_expired_and_duplicate_responses() {
    let ids = runtime_ids();
    let (_temp, registry) = edit_registry();
    let (call, mut pending, _events) = pending_tool_call(ids, &registry);
    let request_id = pending.id;
    pending.expires_at = Some(ts(1));
    let mut coordinator = ApprovalCoordinator::default();
    let mut effects = InMemoryEventStore::default();

    coordinator
        .register_suspended_tool(ids, 0, call.clone(), pending.clone())
        .expect("register suspended approval");

    let expired = coordinator
        .resume_tool_approval(
            ApprovalResponse::allow(request_id, "late approval", Some(60), ts(2))
                .expect("response"),
            &registry,
            &mut effects,
        )
        .expect_err("expired approval rejected");

    assert_eq!(expired, ApprovalResumeError::Expired(request_id));

    let mut coordinator = ApprovalCoordinator::default();
    let mut fresh = pending;
    fresh.expires_at = None;
    coordinator
        .register_suspended_tool(ids, 0, call, fresh)
        .expect("register suspended approval");
    coordinator
        .resume_tool_approval(
            ApprovalResponse::allow(request_id, "approved", Some(60), ts(3)).expect("response"),
            &registry,
            &mut effects,
        )
        .expect("first approval");

    let duplicate = coordinator
        .resume_tool_approval(
            ApprovalResponse::allow(request_id, "approved again", Some(60), ts(4))
                .expect("response"),
            &registry,
            &mut effects,
        )
        .expect_err("duplicate approval rejected");

    assert_eq!(duplicate, ApprovalResumeError::StaleRequest(request_id));
}

#[test]
fn approval_resume_replays_completed_effect_without_duplicate_mutation() {
    let ids = runtime_ids();
    let (temp, registry) = edit_registry();
    let (call, pending, _events) = pending_tool_call(ids, &registry);
    let request_id = pending.id;
    let mut effects = InMemoryEventStore::default();

    let mut first = ApprovalCoordinator::default();
    first
        .register_suspended_tool(ids, 0, call.clone(), pending.clone())
        .expect("register first");
    first
        .resume_tool_approval(
            ApprovalResponse::allow(request_id, "approved", Some(60), ts(1)).expect("response"),
            &registry,
            &mut effects,
        )
        .expect("first resume");
    fs::write(temp.path().join("workspace/README.md"), "old text\n").expect("reset file");

    let mut recovered = ApprovalCoordinator::default();
    recovered
        .register_suspended_tool(ids, 0, call, pending)
        .expect("register recovered");
    let replayed = recovered
        .resume_tool_approval(
            ApprovalResponse::allow(request_id, "approved after restart", Some(60), ts(2))
                .expect("response"),
            &registry,
            &mut effects,
        )
        .expect("replay resume");

    assert_eq!(replayed.status, ApprovalResumeStatus::Executed);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n",
        "idempotent replay must not mutate after the completed effect was recorded"
    );
    assert!(
        replayed
            .tool
            .expect("tool outcome")
            .events
            .iter()
            .any(|event| {
                matches!(
                    event.payload,
                    RuntimeEventPayload::ToolObservationRecorded(_)
                )
            })
    );
}

#[test]
fn approval_recovery_lists_unresolved_pending_approvals_from_events() {
    let ids = runtime_ids();
    let (_temp, registry) = edit_registry();
    let (_call, pending, events) = pending_tool_call(ids, &registry);
    let mut store = InMemoryEventStore::default();

    for event in events {
        store.append(event).expect("append event");
    }

    let stored = store
        .list_by_conversation(ids.conversation_id)
        .expect("stored events");
    let pending_views =
        ApprovalRecoveryView::pending_from_events(stored.iter().map(|stored| &stored.event));

    assert_eq!(pending_views.len(), 1);
    assert_eq!(pending_views[0].request_id, pending.id);
    assert_eq!(
        pending_views[0].checkpoint_id,
        pending.resume_token.map(|token| token.checkpoint_id)
    );
}
