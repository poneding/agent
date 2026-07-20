use agent_core::ConversationCompactionReport;
use agent_types::ConversationId;
use agent_ui_bridge::{UiConversationCompactionReport, UiJsonFrame, encode_json_frame};

#[test]
fn conversation_compaction_frame_serializes_redacted_bounded_summary() {
    let report = ConversationCompactionReport {
        conversation_id: ConversationId::new(),
        source_message_count: 3,
        summary_message_count: 1,
        dropped_message_count: 3,
        estimated_tokens: 32,
        summary: Some(format!(
            "keep this but redact token sk-secret {}",
            "x".repeat(3_000)
        )),
    };

    let frame = UiConversationCompactionReport::from_report(&report);

    assert_eq!(frame.source_message_count, 3);
    assert!(
        frame
            .summary
            .as_ref()
            .expect("summary")
            .contains("[redacted]")
    );
    assert!(
        !frame
            .summary
            .as_ref()
            .expect("summary")
            .contains("sk-secret")
    );
    assert!(frame.summary.as_ref().expect("summary").chars().count() <= 2_000);

    let line = encode_json_frame(&UiJsonFrame::conversation_compaction(frame)).expect("encode");
    let decoded: UiJsonFrame = serde_json::from_str(line.trim_end()).expect("decode");
    assert!(matches!(decoded, UiJsonFrame::conversation_compaction(_)));
}
