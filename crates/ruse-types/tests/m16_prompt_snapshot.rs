use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, PromptLayerHashRecord,
    PromptSnapshotRecorded, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, TurnId,
    WorkspaceId,
};
use serde_json::json;

#[test]
fn m16_prompt_snapshot_event_serializes_stable_shape() {
    let event = RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::from_uuid(uuid::Uuid::nil()),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
        conversation_id: ConversationId::from_uuid(uuid::Uuid::nil()),
        turn_id: Some(TurnId::from_uuid(uuid::Uuid::nil())),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
        causation_id: None,
        payload: RuntimeEventPayload::PromptSnapshotRecorded(PromptSnapshotRecorded {
            profile_id: "default".to_string(),
            profile_version: 1,
            bundle_hash: "fnv1a64:bundle".to_string(),
            layer_hashes: vec![PromptLayerHashRecord {
                layer_id: "system".to_string(),
                source_hash: "fnv1a64:source".to_string(),
                content_hash: "fnv1a64:content".to_string(),
            }],
            diagnostic_count: 0,
            message_count: 1,
            estimated_tokens: 5,
        }),
    };

    let value = serde_json::to_value(event).expect("serialize event");

    assert_eq!(
        value["payload"],
        json!({
            "type": "prompt_snapshot_recorded",
            "data": {
                "profile_id": "default",
                "profile_version": 1,
                "bundle_hash": "fnv1a64:bundle",
                "layer_hashes": [
                    {
                        "layer_id": "system",
                        "source_hash": "fnv1a64:source",
                        "content_hash": "fnv1a64:content"
                    }
                ],
                "diagnostic_count": 0,
                "message_count": 1,
                "estimated_tokens": 5
            }
        })
    );
}
