use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, FinalResponseProduced,
    ModelReasoningSummaryDelta, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp,
    TurnId, UserInputReceived, WorkspaceId,
};
use time::macros::datetime;
use uuid::Uuid;

fn fixed_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::from_uuid(Uuid::from_u128(1)),
        seq: None,
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        workspace_id: WorkspaceId::from_uuid(Uuid::from_u128(2)),
        conversation_id: ConversationId::from_uuid(Uuid::from_u128(3)),
        turn_id: Some(TurnId::from_uuid(Uuid::from_u128(4))),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::from_uuid(Uuid::from_u128(5)),
        causation_id: None,
        payload,
    }
}

#[test]
fn user_input_event_round_trips_through_json() {
    let event = fixed_event(RuntimeEventPayload::UserInputReceived(UserInputReceived {
        message: "inspect the workspace".to_string(),
    }));

    let json = serde_json::to_string_pretty(&event).expect("serialize event");
    let decoded: RuntimeEvent = serde_json::from_str(&json).expect("deserialize event");

    assert_eq!(decoded, event);
    assert!(json.contains("\"type\": \"user_input_received\""));
}

#[test]
fn final_response_event_has_stable_shape() {
    let event = fixed_event(RuntimeEventPayload::FinalResponseProduced(
        FinalResponseProduced {
            message: "done".to_string(),
        },
    ));

    let value = serde_json::to_value(event).expect("event json value");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["payload"]["type"], "final_response_produced");
    assert_eq!(value["payload"]["data"]["message"], "done");
    assert_eq!(value["occurred_at"], "2026-06-25T00:00:00Z");
}

#[test]
fn reasoning_summary_delta_event_has_stable_shape() {
    let event = fixed_event(RuntimeEventPayload::ModelReasoningSummaryDelta(
        ModelReasoningSummaryDelta {
            delta: "checking context".to_string(),
        },
    ));

    let value = serde_json::to_value(event).expect("event json value");

    assert_eq!(value["payload"]["type"], "model_reasoning_summary_delta");
    assert_eq!(value["payload"]["data"]["delta"], "checking context");
}
