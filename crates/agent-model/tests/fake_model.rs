use agent_model::{
    FakeModelProvider, FakeModelScript, ModelProvider, ModelRequest, ModelStreamEvent,
    collect_model_event_stream,
};
use agent_types::{ConversationId, CorrelationId, TurnId};

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec!["hello".to_string()],
    }
}

#[tokio::test]
async fn fake_model_streams_scripted_deltas() {
    let provider = FakeModelProvider::new(FakeModelScript {
        request_id: "request-42".to_string(),
        deltas: vec!["hel".to_string(), "lo".to_string()],
        tool_calls: Vec::new(),
        finish_reason: "stop".to_string(),
    });

    let stream = provider
        .stream(model_request())
        .await
        .expect("stream fake model");
    let events = collect_model_event_stream(stream)
        .await
        .expect("collect stream");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "request-42".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "hel".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "lo".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 5,
            },
        ]
    );
}

#[tokio::test]
async fn fake_model_empty_script_emits_started_and_completed() {
    let provider = FakeModelProvider::new(FakeModelScript {
        request_id: "empty-request".to_string(),
        deltas: Vec::new(),
        tool_calls: Vec::new(),
        finish_reason: "stop".to_string(),
    });

    let stream = provider
        .stream(model_request())
        .await
        .expect("stream fake model");
    let events = collect_model_event_stream(stream)
        .await
        .expect("collect stream");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "empty-request".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 0,
            },
        ]
    );
}

#[test]
fn fake_model_provider_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<FakeModelProvider>();
}

#[test]
fn model_request_preserves_metadata() {
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let correlation_id = CorrelationId::new();

    let request = ModelRequest {
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
        conversation_id,
        turn_id,
        correlation_id,
        messages: vec!["first".to_string(), "second".to_string()],
    };

    assert_eq!(request.provider, "fake");
    assert_eq!(request.model, "fake-scripted");
    assert_eq!(request.conversation_id, conversation_id);
    assert_eq!(request.turn_id, turn_id);
    assert_eq!(request.correlation_id, correlation_id);
    assert_eq!(request.messages, vec!["first", "second"]);
}
