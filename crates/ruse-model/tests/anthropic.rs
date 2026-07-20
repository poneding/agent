use std::sync::Arc;

use ruse_model::{
    AnthropicError, AnthropicHttpResponse, AnthropicProvider, FixtureAnthropicTransport,
    ModelEndpointConfig, ModelError, ModelMessage, ModelProvider, ModelProviderConfig,
    ModelProviderKind, ModelRequest, ModelStreamEvent, ModelToolDefinition, SecretRef,
    build_anthropic_request_body, collect_model_event_stream, parse_anthropic_sse,
};
use ruse_types::{ConversationId, CorrelationId, ReasoningEffort, TurnId};
use serde_json::json;

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "anthropic".to_string(),
        model: "claude-3-5-sonnet-latest".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec![
            ModelMessage::system("follow project instructions"),
            ModelMessage::user("summarize this"),
        ],
        reasoning_effort: None,
        tools: Vec::new(),
    }
}

fn tool_request() -> ModelRequest {
    let mut request = model_request();
    request.messages.extend([
        ModelMessage::assistant_tool_call(
            "call-1",
            "workspace.read_file",
            json!({ "path": "README.md" }),
        ),
        ModelMessage::tool_result("call-1", "workspace.read_file", "hello from tool"),
    ]);
    request.tools = vec![ModelToolDefinition {
        name: "workspace.read_file".to_string(),
        description: "Read a workspace file".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string" }
            },
            "additionalProperties": false
        }),
    }];
    request
}

fn secret_ref() -> SecretRef {
    SecretRef::new("env", "ANTHROPIC_API_KEY").expect("valid secret ref")
}

fn provider_config(api_key: Option<SecretRef>) -> ModelProviderConfig {
    ModelProviderConfig {
        id: "anthropic".to_string(),
        kind: ModelProviderKind::Anthropic,
        default_model: "claude-3-5-sonnet-latest".to_string(),
        endpoint: ModelEndpointConfig::new(Some("https://api.anthropic.com/v1"), 30_000)
            .expect("valid endpoint"),
        api_key,
        enabled: true,
    }
}

#[test]
fn anthropic_request_builder_emits_messages_payload() {
    let body = build_anthropic_request_body(&model_request());

    assert_eq!(
        body,
        json!({
            "model": "claude-3-5-sonnet-latest",
            "system": "follow project instructions",
            "messages": [
                {
                    "role": "user",
                    "content": [{ "type": "text", "text": "summarize this" }]
                }
            ],
            "max_tokens": 1024,
            "stream": true
        })
    );
}

#[test]
fn anthropic_request_builder_emits_reasoning_effort() {
    let mut request = model_request();
    request.reasoning_effort = Some(ReasoningEffort::XHigh);

    let body = build_anthropic_request_body(&request);

    assert_eq!(body["output_config"], json!({ "effort": "max" }));
}

#[test]
fn anthropic_request_builder_emits_tools_payload() {
    let body = build_anthropic_request_body(&tool_request());

    assert_eq!(body["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(body["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(
        body["tools"],
        json!([
            {
                "name": "workspace.read_file",
                "description": "Read a workspace file",
                "input_schema": {
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        ])
    );
}

#[test]
fn anthropic_provider_requires_secret_ref() {
    let err = AnthropicProvider::new(
        provider_config(None),
        Arc::new(FixtureAnthropicTransport::with_chunks(
            "request-1",
            Vec::<String>::new(),
        )),
    )
    .expect_err("missing secret ref rejected");

    assert!(matches!(
        err,
        AnthropicError::MissingSecretRef { provider_id } if provider_id == "anthropic"
    ));
}

#[test]
fn anthropic_provider_rejects_invalid_provider_kind() {
    let mut config = provider_config(Some(secret_ref()));
    config.kind = ModelProviderKind::OpenAiCompatible;

    let err = AnthropicProvider::new(
        config,
        Arc::new(FixtureAnthropicTransport::with_chunks(
            "request-1",
            Vec::<String>::new(),
        )),
    )
    .expect_err("invalid provider kind rejected");

    assert!(matches!(
        err,
        AnthropicError::InvalidProviderKind { provider_id } if provider_id == "anthropic"
    ));
}

#[tokio::test]
async fn anthropic_stream_parser_maps_delta_and_completed_events() {
    let provider = AnthropicProvider::new(
        provider_config(Some(secret_ref())),
        Arc::new(FixtureAnthropicTransport::with_chunks(
            "request-1",
            vec![
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\"}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hel\"}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}\n\n".to_string(),
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n".to_string(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            ],
        )),
    )
    .expect("valid provider");

    let stream = provider
        .stream(model_request())
        .await
        .expect("stream fixture response");
    let events = collect_model_event_stream(stream)
        .await
        .expect("collect stream");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "request-1".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "hel".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "lo".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "end_turn".to_string(),
                output_token_count: 2,
            },
        ]
    );
}

#[test]
fn anthropic_stream_parser_maps_tool_use_blocks() {
    let events = parse_anthropic_sse(
        "request-1",
        &AnthropicHttpResponse {
            request_id: "request-1".to_string(),
            sse_chunks: vec![
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"read_file\",\"input\":{}}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\":\"}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"README.md\\\"}\"}}\n\n".to_string(),
                "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":1}\n\n".to_string(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            ],
        },
    )
    .expect("parse tool use");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "request-1".to_string(),
            },
            ModelStreamEvent::ToolCall {
                call_id: "toolu_1".to_string(),
                tool_name: "read_file".to_string(),
                input_json: json!({ "path": "README.md" }),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 0,
            },
        ]
    );
}

#[test]
fn anthropic_errors_redact_response_bodies_and_classify_retryability() {
    let err = AnthropicError::provider_response(
        401,
        "{\"error\":\"invalid key sk-ant-test-secret api_key token should not leak\"}",
    );
    let model_error: ModelError = err.into();
    let rendered = model_error.to_string();

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-ant-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(matches!(
        model_error,
        ModelError::Provider {
            code,
            retryable: false,
            ..
        } if code == "model.anthropic.provider_response"
    ));
    assert!(AnthropicError::provider_response(529, "overloaded").retryable());
}
