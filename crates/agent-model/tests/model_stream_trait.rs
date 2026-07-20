use async_trait::async_trait;
use futures_util::stream;

use agent_model::{
    FakeModelProvider, FakeModelScript, ModelError, ModelEventStream, ModelMessage, ModelProvider,
    ModelRequest, ModelStreamEvent, collect_model_event_stream,
};
use agent_types::{ConversationId, CorrelationId, TurnId};

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec![ModelMessage::user("hello")],
        reasoning_effort: None,
        tools: Vec::new(),
    }
}

#[tokio::test]
async fn model_event_stream_collects_fake_events_in_order() {
    let provider = FakeModelProvider::new(FakeModelScript::text("hello"));

    let stream = provider.stream(model_request()).await.expect("stream");
    let events = collect_model_event_stream(stream).await.expect("collect");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "fake-request-1".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "hello".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 5,
            },
        ]
    );
}

#[tokio::test]
async fn model_event_stream_collects_reasoning_summary_events() {
    let stream = Box::pin(stream::iter(vec![
        Ok(ModelStreamEvent::Started {
            request_id: "request-1".to_string(),
        }),
        Ok(ModelStreamEvent::ReasoningSummaryDelta {
            text: "checking fixture".to_string(),
        }),
        Ok(ModelStreamEvent::Delta {
            text: "answer".to_string(),
        }),
    ]));

    let events = collect_model_event_stream(stream).await.expect("collect");

    assert!(matches!(
        &events[1],
        ModelStreamEvent::ReasoningSummaryDelta { text } if text == "checking fixture"
    ));
}

#[derive(Debug)]
struct ErrorAfterStartProvider;

#[async_trait]
impl ModelProvider for ErrorAfterStartProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        Ok(Box::pin(stream::iter(vec![
            Ok(ModelStreamEvent::Started {
                request_id: "request-1".to_string(),
            }),
            Err(ModelError::Script("boom".to_string())),
        ])))
    }
}

#[tokio::test]
async fn model_event_stream_propagates_item_errors() {
    let provider = ErrorAfterStartProvider;

    let stream = provider.stream(model_request()).await.expect("stream");
    let error = collect_model_event_stream(stream)
        .await
        .expect_err("item error propagates");

    assert_eq!(error, ModelError::Script("boom".to_string()));
}
