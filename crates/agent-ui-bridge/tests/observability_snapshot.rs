use agent_core::{
    ObservabilityEvidenceBundle, ObservabilityEvidenceOptions, build_observability_evidence,
};
use agent_types::{
    ActorRef, CheckpointId, CheckpointRecorded, ConversationId, CorrelationId,
    FinalResponseProduced, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp,
    ToolCallProposed, TurnId, WorkspaceId,
};
use agent_ui_bridge::{UiJsonFrame, UiObservabilitySnapshot, encode_json_frame};
use serde_json::json;

fn runtime_event(conversation_id: ConversationId, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id,
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

fn evidence() -> ObservabilityEvidenceBundle {
    let conversation_id = ConversationId::new();
    let events = vec![
        runtime_event(
            conversation_id,
            RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
                call_id: "call-read".to_string(),
                tool_name: "workspace.read_file".to_string(),
                input_json: json!({ "path": "README.md" }),
            }),
        ),
        runtime_event(
            conversation_id,
            RuntimeEventPayload::CheckpointRecorded(CheckpointRecorded {
                checkpoint_id: CheckpointId::new(),
                label: "before final".to_string(),
            }),
        ),
        runtime_event(
            conversation_id,
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: "done".to_string(),
            }),
        ),
    ];

    build_observability_evidence(&events, &[], ObservabilityEvidenceOptions::default())
        .expect("build evidence")
}

#[test]
fn observability_snapshot_serializes_stable_shape() {
    let snapshot = UiObservabilitySnapshot::from_evidence(&evidence()).expect("snapshot");

    assert_eq!(
        serde_json::to_value(snapshot).expect("serialize snapshot"),
        json!({
            "schema_version": 1,
            "event_count": 3,
            "trace_count": 3,
            "audit_count": 3,
            "telemetry_count": 0,
            "replayable": true,
            "diagnostics": []
        })
    );
}

#[test]
fn observability_snapshot_bounds_and_redacts_diagnostics() {
    let mut evidence = evidence();
    evidence
        .diagnostics
        .push(format!("token sk-secret-value {}", "x".repeat(180)));

    let snapshot = UiObservabilitySnapshot::from_evidence(&evidence).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert_eq!(snapshot.diagnostics.len(), 1);
    assert!(snapshot.diagnostics[0].chars().count() <= 120);
    assert!(!serialized.contains("sk-secret-value"));
    assert!(!serialized.contains("token"));
    assert!(serialized.contains("[redacted]"));
}

#[test]
fn observability_snapshot_json_frame_is_single_line() {
    let snapshot = UiObservabilitySnapshot::from_evidence(&evidence()).expect("snapshot");
    let line = encode_json_frame(&UiJsonFrame::observability_snapshot(snapshot)).expect("frame");
    let value: serde_json::Value = serde_json::from_str(line.trim()).expect("parse frame");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    assert_eq!(value["type"], "observability_snapshot");
}

#[test]
fn observability_snapshot_does_not_expose_raw_evidence_payloads() {
    let conversation_id = ConversationId::new();
    let events = vec![runtime_event(
        conversation_id,
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: "call-secret".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({
                "path": "README.md",
                "api_key": "secret-value",
                "nested": { "token": "another-secret" }
            }),
        }),
    )];
    let evidence = build_observability_evidence(
        &events,
        &[],
        ObservabilityEvidenceOptions {
            include_telemetry: false,
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect("build evidence");

    let snapshot = UiObservabilitySnapshot::from_evidence(&evidence).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("secret-value"));
    assert!(!serialized.contains("another-secret"));
    assert!(!serialized.contains("api_key"));
}
