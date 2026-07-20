use ruse_types::{
    ConversationId, CorrelationId, EventId, EventSeq, SchemaVersion, Timestamp, TurnId,
};
use ruse_ui_bridge::{UiEventEnvelope, UiEventPayload, UiJsonFrame, encode_json_frame};
use serde_json::json;
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn event() -> UiEventEnvelope {
    UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: id::<EventId>("00000000-0000-0000-0000-000000000005"),
        seq: Some(EventSeq::new(7).expect("valid sequence")),
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        turn_id: Some(id::<TurnId>("00000000-0000-0000-0000-000000000003")),
        task_id: None,
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000004"),
        payload: UiEventPayload::ModelDelta {
            delta: "hello".to_string(),
        },
    }
}

#[test]
fn json_frame_serializes_event_with_type_tag() {
    let value = serde_json::to_value(UiJsonFrame::Event(event())).expect("serialize frame");

    assert_eq!(
        value,
        json!({
            "type": "event",
            "data": {
                "schema_version": 1,
                "event_id": "00000000-0000-0000-0000-000000000005",
                "seq": 7,
                "occurred_at": "2026-06-25T00:00:00Z",
                "conversation_id": "00000000-0000-0000-0000-000000000002",
                "turn_id": "00000000-0000-0000-0000-000000000003",
                "task_id": null,
                "correlation_id": "00000000-0000-0000-0000-000000000004",
                "payload": {
                    "type": "model_delta",
                    "data": {
                        "delta": "hello"
                    }
                }
            }
        })
    );
}

#[test]
fn json_frame_serializes_done_count() {
    let value =
        serde_json::to_value(UiJsonFrame::Done { event_count: 3 }).expect("serialize frame");

    assert_eq!(
        value,
        json!({
            "type": "done",
            "data": {
                "event_count": 3
            }
        })
    );
}

#[test]
fn json_frame_encode_returns_single_line() {
    let line = encode_json_frame(&UiJsonFrame::Done { event_count: 3 }).expect("encode frame");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("line is a frame");
}
