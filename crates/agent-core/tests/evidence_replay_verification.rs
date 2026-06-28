use agent_core::{
    EvidenceReplayVerificationExpectation, verify_observability_evidence_replay,
    verify_observability_replay_fixture,
};
use agent_storage::load_replay_fixture_json;
use agent_types::{
    ActorRef, ConversationId, CorrelationId, RuntimeEvent, RuntimeEventPayload, SchemaVersion,
    Timestamp, ToolCallProposed, TurnId, WorkspaceId,
};
use serde_json::json;

const M04_TRACE_V1: &str = include_str!("../../agent-storage/tests/fixtures/m04_trace_v1.json");

fn secret_event() -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload: RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: "call-secret".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({
                "path": "README.md",
                "api_key": "secret-value",
                "nested": { "token": "another-secret" }
            }),
        }),
    }
}

#[test]
fn evidence_replay_verification_regenerates_m04_fixture() {
    let fixture = load_replay_fixture_json(M04_TRACE_V1).expect("load fixture");

    let report = verify_observability_replay_fixture(&fixture).expect("verify fixture evidence");

    assert!(report.replayable);
    assert_eq!(report.source_event_count, fixture.expected.event_count);
    assert_eq!(
        report.source_checkpoint_count,
        fixture.expected.checkpoint_count
    );
    assert!(report.regenerated_trace_count > 0);
    assert_eq!(report.regenerated_audit_count, fixture.expected.event_count);
    assert!(report.diagnostics.is_empty());
}

#[test]
fn evidence_replay_verification_is_idempotent_for_local_coding_fixture() {
    let run = agent_core::run_local_coding_fixture().expect("local coding fixture");
    let expectation = EvidenceReplayVerificationExpectation {
        event_count: Some(run.events.len()),
        checkpoint_count: None,
        audit_record_count: Some(run.events.len()),
    };

    let first =
        verify_observability_evidence_replay(&run.events, expectation).expect("first verify");
    let second =
        verify_observability_evidence_replay(&run.events, expectation).expect("second verify");

    assert_eq!(first, second);
    assert_eq!(first.source_event_count, run.events.len());
    assert_eq!(first.regenerated_audit_count, run.events.len());
}

#[test]
fn evidence_replay_verification_reports_mismatches_without_raw_payloads() {
    let event = secret_event();
    let report = verify_observability_evidence_replay(
        &[event],
        EvidenceReplayVerificationExpectation {
            event_count: Some(999),
            checkpoint_count: Some(10),
            audit_record_count: Some(999),
        },
    )
    .expect("verify evidence");

    assert!(!report.replayable);
    assert!(report.diagnostics.iter().any(|item| item.contains("event")));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| item.contains("checkpoint"))
    );
    assert!(report.diagnostics.iter().any(|item| item.contains("audit")));

    let serialized = serde_json::to_string(&report).expect("serialize report");
    assert!(!serialized.contains("secret-value"));
    assert!(!serialized.contains("another-secret"));
    assert!(!serialized.contains("api_key"));
}
