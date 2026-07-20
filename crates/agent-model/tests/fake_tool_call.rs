use agent_model::{
    FakeModelProvider, FakeModelScript, ModelMessage, ModelProvider, ModelRequest,
    ModelStreamEvent, collect_model_event_stream,
};
use agent_types::{ConversationId, CorrelationId, TurnId};
use serde_json::json;

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec![ModelMessage::user("read README")],
        reasoning_effort: None,
        tools: Vec::new(),
    }
}

#[tokio::test]
async fn fake_model_emits_scripted_tool_call() {
    let provider = FakeModelProvider::new(FakeModelScript::tool_call(
        "call-1",
        "workspace.read_file",
        json!({ "path": "README.md" }),
    ));

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
                request_id: "fake-request-1".to_string(),
            },
            ModelStreamEvent::ToolCall {
                call_id: "call-1".to_string(),
                tool_name: "workspace.read_file".to_string(),
                input_json: json!({ "path": "README.md" }),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 0,
            },
        ]
    );
}
