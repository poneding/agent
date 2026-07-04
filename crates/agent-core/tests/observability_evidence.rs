use agent_core::{
    ObservabilityEvidenceError, ObservabilityEvidenceOptions, build_observability_evidence,
    run_local_coding_fixture,
};
use agent_telemetry::{
    InMemoryTelemetrySink, TelemetryAttribute, TelemetryExportOptions, TelemetryMetric,
    TelemetrySink, TelemetrySpan, TraceContext,
};
use agent_types::{
    ActorRef, CheckpointId, CheckpointRecorded, ConversationId, CorrelationId,
    FinalResponseProduced, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp,
    ToolCallProposed, TurnId, WorkspaceId,
};
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

fn telemetry_records() -> Vec<agent_telemetry::TelemetryRecord> {
    let mut sink = InMemoryTelemetrySink::default();
    sink.record_span(
        TelemetrySpan::new(
            "turn.execute",
            TraceContext::new("trace-1", "span-1", None).expect("valid context"),
            vec![TelemetryAttribute::new("workspace_id", "workspace-1").expect("attribute")],
        )
        .expect("valid span"),
    );
    sink.record_metric(
        TelemetryMetric::new("tool.duration", "ms", 7.0, Vec::new()).expect("valid metric"),
    );
    sink.records().to_vec()
}

#[test]
fn observability_evidence_serializes_stable_shape() {
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

    let bundle = build_observability_evidence(
        &events,
        &telemetry_records(),
        ObservabilityEvidenceOptions::default(),
    )
    .expect("build evidence");
    let value = serde_json::to_value(bundle).expect("serialize evidence");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["summary"]["total_events"], 3);
    assert_eq!(value["summary"]["trace_entries"], 3);
    assert_eq!(value["summary"]["audit_records"], 3);
    assert_eq!(value["summary"]["telemetry_records"], 2);
    assert_eq!(value["summary"]["replayable"], true);
    assert_eq!(value["trace"][0]["kind"], "tool_call");
    assert_eq!(value["audit"]["summary"]["audit_records"], 3);
    assert_eq!(value["telemetry"]["summary"]["exported_records"], 2);
    assert_eq!(value["replay"]["checkpoint_count"], 1);
}

#[test]
fn observability_evidence_summarizes_local_coding_fixture() {
    let run = run_local_coding_fixture().expect("local coding fixture");

    let bundle = build_observability_evidence(
        &run.events,
        &[],
        ObservabilityEvidenceOptions {
            include_telemetry: false,
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect("build evidence");

    assert_eq!(bundle.summary.total_events, run.events.len());
    assert!(bundle.summary.trace_entries > 0);
    assert_eq!(bundle.summary.audit_records, run.events.len());
    assert!(bundle.replay.replayable);
}

#[test]
fn observability_evidence_rejects_invalid_options() {
    let invalid_payload = build_observability_evidence(
        &[],
        &[],
        ObservabilityEvidenceOptions {
            max_payload_bytes: 0,
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect_err("zero payload budget rejected");
    assert_eq!(
        invalid_payload,
        ObservabilityEvidenceError::InvalidMaxPayloadBytes
    );

    let no_sections = build_observability_evidence(
        &[],
        &[],
        ObservabilityEvidenceOptions {
            include_trace: false,
            include_audit: false,
            include_telemetry: false,
            include_replay: false,
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect_err("empty section selection rejected");
    assert_eq!(
        no_sections,
        ObservabilityEvidenceError::NoEvidenceSectionsSelected
    );
}

#[test]
fn observability_evidence_does_not_expose_raw_secrets() {
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
    let mut sink = InMemoryTelemetrySink::default();
    sink.record_span(
        TelemetrySpan::new(
            "tool.call",
            TraceContext::new("trace-secret", "span-secret", None).expect("valid context"),
            vec![
                TelemetryAttribute::new("api_token", "sk-live-secret").expect("secret telemetry"),
                TelemetryAttribute::new("note", "plain value").expect("plain telemetry"),
            ],
        )
        .expect("valid span"),
    );

    let bundle = build_observability_evidence(
        &events,
        sink.records(),
        ObservabilityEvidenceOptions {
            telemetry: TelemetryExportOptions {
                max_attribute_value_bytes: 64,
                ..TelemetryExportOptions::default()
            },
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect("build evidence");
    let serialized = serde_json::to_string(&bundle).expect("serialize evidence");

    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("secret-value"));
    assert!(!serialized.contains("another-secret"));
    assert!(!serialized.contains("sk-live-secret"));
}
