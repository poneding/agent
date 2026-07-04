use agent_storage::{
    CheckpointStore, InMemoryEventStore, ReplayCompatibilityError, ReplayCursor,
    check_replay_compatibility, load_replay_fixture_json,
};
use agent_types::RuntimeEventPayload;
use serde_json::json;

const M04_TRACE_V1: &str = include_str!("fixtures/m04_trace_v1.json");

#[test]
fn replay_compatibility_parses_m04_fixture_and_reports_counts() {
    let fixture = load_replay_fixture_json(M04_TRACE_V1).expect("load fixture");
    let report = check_replay_compatibility(M04_TRACE_V1).expect("check compatibility");

    assert_eq!(fixture.fixture_name, "m04_trace_v1");
    assert_eq!(fixture.events.len(), 9);
    assert_eq!(fixture.checkpoints.len(), 1);
    assert_eq!(
        fixture.expected.final_summary,
        "README inspected with permissioned tool call."
    );
    assert_eq!(report.fixture_name, "m04_trace_v1");
    assert_eq!(report.event_count, 9);
    assert_eq!(report.checkpoint_count, 1);
    assert!(report.compatible);
}

#[test]
fn replay_compatibility_replays_old_trace_events() {
    let fixture = load_replay_fixture_json(M04_TRACE_V1).expect("load fixture");
    let mut store = InMemoryEventStore::default();

    fixture
        .load_into_store(&mut store)
        .expect("load into store");
    let replayed = ReplayCursor::new(fixture.expected.conversation_id)
        .collect(&store)
        .expect("replay fixture");

    assert_eq!(replayed.len(), fixture.expected.event_count);
    assert_eq!(replayed[0].seq.get(), 1);
    assert_eq!(replayed[8].seq.get(), 9);
    assert!(matches!(
        replayed[1].event.payload,
        RuntimeEventPayload::ToolCallProposed(_)
    ));
}

#[test]
fn replay_compatibility_loads_checkpoint_records() {
    let fixture = load_replay_fixture_json(M04_TRACE_V1).expect("load fixture");
    let mut store = InMemoryEventStore::default();

    fixture
        .load_into_store(&mut store)
        .expect("load into store");
    let latest = store
        .latest_checkpoint(
            fixture.expected.conversation_id,
            Some(fixture.expected.turn_id),
        )
        .expect("load checkpoint")
        .expect("checkpoint exists");

    assert_eq!(latest.label, "after read");
    assert_eq!(latest.event_seq.get(), 7);
    assert_eq!(
        latest.state_json["completed_call_ids"],
        json!(["call-readme"])
    );
}

#[test]
fn replay_compatibility_rejects_future_fixture_versions() {
    let fixture = json!({
        "fixture_schema_version": 999,
        "fixture_name": "future_trace",
        "events": [],
        "checkpoints": [],
        "expected": {}
    })
    .to_string();

    let error = load_replay_fixture_json(&fixture).expect_err("future version rejected");

    assert!(matches!(
        error,
        ReplayCompatibilityError::UnsupportedFutureVersion {
            fixture_version: 999,
            supported_version: 1
        }
    ));
    assert!(
        error
            .to_string()
            .contains("unsupported future replay fixture version 999")
    );
}

#[test]
fn replay_compatibility_requires_explicit_fixture_version() {
    let fixture = json!({
        "fixture_name": "missing_version",
        "events": [],
        "checkpoints": [],
        "expected": {}
    })
    .to_string();

    let error = load_replay_fixture_json(&fixture).expect_err("missing version rejected");

    assert!(matches!(
        error,
        ReplayCompatibilityError::MissingField(field) if field == "fixture_schema_version"
    ));
}
