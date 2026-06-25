use agent_model::{
    FakeModelProvider, FakeModelScript, ModelProvider, ModelRequest, ModelStreamEvent,
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
        messages: vec!["read README".to_string()],
    }
}

#[tokio::test]
async fn fake_model_emits_scripted_tool_call() {
    let provider = FakeModelProvider::new(FakeModelScript::tool_call(
        "call-1",
        "workspace.read_file",
        json!({ "path": "README.md" }),
    ));

    let events = provider
        .stream(model_request())
        .await
        .expect("stream fake model");

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
