use agent_core::{TraceQuery, build_trace};
use agent_storage::{InMemoryEventStore, load_replay_fixture_json};

const M04_TRACE_V1: &str = include_str!("../../agent-storage/tests/fixtures/m04_trace_v1.json");

#[test]
fn recovery_compatibility_reconstructs_m04_trace_summary() {
    let fixture = load_replay_fixture_json(M04_TRACE_V1).expect("load fixture");
    let mut store = InMemoryEventStore::default();
    fixture
        .load_into_store(&mut store)
        .expect("load into store");

    let trace = build_trace(
        &store,
        TraceQuery {
            conversation_id: fixture.expected.conversation_id,
            include_tools: true,
            include_policy: true,
            include_checkpoints: true,
            include_final: true,
        },
    )
    .expect("build trace");
    let final_summary = trace
        .iter()
        .find(|entry| entry.kind == "turn_completed")
        .map(|entry| entry.summary.as_str());
    let audit_count = trace
        .iter()
        .filter(|entry| {
            matches!(
                entry.kind.as_str(),
                "tool_call"
                    | "tool_start"
                    | "tool_result"
                    | "tool_error"
                    | "tool_observation"
                    | "permission"
            )
        })
        .count();

    assert_eq!(
        final_summary,
        Some("turn completed Done: README inspected with permissioned tool call.")
    );
    assert_eq!(audit_count, fixture.expected.audit_event_count);
}
