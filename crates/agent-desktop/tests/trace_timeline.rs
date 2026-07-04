use agent_core::TraceEntry;
use agent_desktop::{
    DesktopError, DesktopRuntimeLogSeverity, TraceTimelineSeverity, build_trace_timeline,
};
use agent_types::ConversationId;

fn conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000000001"
        .parse()
        .expect("valid conversation id")
}

fn entry(seq: u64, kind: &str, summary: &str) -> TraceEntry {
    TraceEntry {
        seq,
        kind: kind.to_string(),
        summary: summary.to_string(),
    }
}

#[test]
fn trace_timeline_serializes_timeline_shape() {
    let timeline = build_trace_timeline(
        conversation_id(),
        vec![entry(7, "tool_error", "failed shell recoverable=true")],
    )
    .unwrap();

    let value = serde_json::to_value(timeline).expect("serialize trace timeline");

    assert_eq!(
        value["conversation_id"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(value["items"][0]["seq"], 7);
    assert_eq!(value["items"][0]["kind"], "tool_error");
    assert_eq!(value["items"][0]["title"], "Tool Error");
    assert_eq!(
        value["items"][0]["summary"],
        "failed shell recoverable=true"
    );
    assert_eq!(value["items"][0]["severity"], "error");
    assert_eq!(value["logs"][0]["sequence"], 7);
    assert_eq!(value["logs"][0]["severity"], "error");
    assert_eq!(value["logs"][0]["source"], "tool_error");
}

#[test]
fn trace_timeline_sorts_by_sequence() {
    let timeline = build_trace_timeline(
        conversation_id(),
        vec![
            entry(3, "final", "final response: done"),
            entry(1, "checkpoint", "checkpoint saved"),
            entry(2, "permission", "allow: user approved"),
        ],
    )
    .unwrap();

    let seqs = timeline
        .items
        .iter()
        .map(|item| item.seq)
        .collect::<Vec<_>>();

    assert_eq!(seqs, vec![1, 2, 3]);
    assert_eq!(timeline.items[0].severity, TraceTimelineSeverity::Info);
    assert_eq!(timeline.logs[0].sequence, 1);
}

#[test]
fn trace_timeline_rejects_duplicate_sequences() {
    let err = build_trace_timeline(
        conversation_id(),
        vec![entry(1, "checkpoint", "saved"), entry(1, "final", "done")],
    )
    .unwrap_err();

    assert!(matches!(err, DesktopError::InvalidTrace(_)));
}

#[test]
fn trace_timeline_rejects_empty_kind_and_summary() {
    let empty_kind = build_trace_timeline(conversation_id(), vec![entry(1, " ", "summary")]);
    let empty_summary = build_trace_timeline(conversation_id(), vec![entry(1, "final", " ")]);

    assert!(matches!(empty_kind, Err(DesktopError::InvalidTrace(_))));
    assert!(matches!(empty_summary, Err(DesktopError::InvalidTrace(_))));
}

#[test]
fn trace_timeline_does_not_expose_secret_like_fields() {
    let timeline = build_trace_timeline(
        conversation_id(),
        vec![entry(
            1,
            "tool_call",
            "input {\"api_key\":\"sk-test\",\"token\":\"abc\"}",
        )],
    )
    .unwrap();

    let serialized = serde_json::to_string(&timeline).expect("serialize trace timeline");

    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("sk-test"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("abc"));
}

#[test]
fn trace_timeline_builds_bounded_runtime_logs() {
    let timeline = build_trace_timeline(
        conversation_id(),
        vec![
            entry(2, "runtime_warning", "warning happened"),
            entry(
                1,
                "tool_error",
                &format!("token sk-secret {}", "x".repeat(400)),
            ),
        ],
    )
    .unwrap();
    let serialized = serde_json::to_string(&timeline.logs).expect("serialize logs");

    assert_eq!(timeline.logs[0].sequence, 1);
    assert_eq!(timeline.logs[0].severity, DesktopRuntimeLogSeverity::Error);
    assert_eq!(
        timeline.logs[1].severity,
        DesktopRuntimeLogSeverity::Warning
    );
    assert!(timeline.logs[0].truncated);
    assert!(timeline.logs[0].message.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
}
