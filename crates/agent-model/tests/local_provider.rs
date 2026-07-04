use std::sync::Arc;

use agent_model::{
    FixtureLocalTransport, LocalError, LocalHttpResponse, LocalProvider, ModelEndpointConfig,
    ModelError, ModelProvider, ModelProviderConfig, ModelProviderKind, ModelRequest,
    ModelStreamEvent, build_local_request_body, collect_model_event_stream, local_chat_url,
    parse_local_jsonl,
};
use agent_types::{ConversationId, CorrelationId, TurnId};
use serde_json::json;

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "local".to_string(),
        model: "llama3.1".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec!["hello".to_string(), "summarize this".to_string()],
    }
}

fn provider_config(base_url: Option<&str>) -> ModelProviderConfig {
    ModelProviderConfig {
        id: "local".to_string(),
        kind: ModelProviderKind::Local,
        default_model: "llama3.1".to_string(),
        endpoint: ModelEndpointConfig::new(base_url, 30_000).expect("valid endpoint"),
        api_key: None,
        enabled: true,
    }
}

#[test]
fn local_provider_request_builder_emits_chat_payload() {
    let body = build_local_request_body(&model_request());

    assert_eq!(
        body,
        json!({
            "model": "llama3.1",
            "messages": [
                { "role": "user", "content": "hello" },
                { "role": "user", "content": "summarize this" }
            ],
            "stream": true
        })
    );
}

#[test]
fn local_provider_url_builder_appends_chat_endpoint() {
    let url = local_chat_url(&provider_config(Some("http://localhost:11434")));

    assert_eq!(url, "http://localhost:11434/api/chat");
    assert!(!url.contains('@'));
    assert!(!url.contains('?'));
}

#[test]
fn local_provider_accepts_local_config_without_secret() {
    let provider = LocalProvider::new(
        provider_config(Some("http://127.0.0.1:11434")),
        Arc::new(FixtureLocalTransport::with_chunks(
            "request-1",
            Vec::<String>::new(),
        )),
    )
    .expect("valid local provider");

    assert_eq!(provider.config().id, "local");
}

#[test]
fn local_provider_rejects_invalid_provider_kind() {
    let mut config = provider_config(Some("http://localhost:11434"));
    config.kind = ModelProviderKind::Gemini;

    let err = LocalProvider::new(
        config,
        Arc::new(FixtureLocalTransport::with_chunks(
            "request-1",
            Vec::<String>::new(),
        )),
    )
    .expect_err("invalid provider kind rejected");

    assert!(matches!(
        err,
        LocalError::InvalidProviderKind { provider_id } if provider_id == "local"
    ));
}

#[tokio::test]
async fn local_provider_stream_parser_maps_delta_and_completed_events() {
    let provider = LocalProvider::new(
        provider_config(Some("http://localhost:11434")),
        Arc::new(FixtureLocalTransport::with_chunks(
            "request-1",
            vec![
                "{\"message\":{\"content\":\"hel\"},\"done\":false}\n".to_string(),
                "{\"message\":{\"content\":\"lo\"},\"done\":false}\n{\"done\":true,\"done_reason\":\"stop\",\"eval_count\":2}\n".to_string(),
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
                finish_reason: "stop".to_string(),
                output_token_count: 2,
            },
        ]
    );
}

#[test]
fn local_provider_stream_parser_maps_tool_calls() {
    let events = parse_local_jsonl(
        "request-1",
        &LocalHttpResponse {
            request_id: "request-1".to_string(),
            jsonl_chunks: vec![
                "{\"message\":{\"tool_calls\":[{\"function\":{\"name\":\"read_file\",\"arguments\":{\"path\":\"README.md\"}}}]},\"done\":false}\n".to_string(),
                "{\"done\":true,\"done_reason\":\"stop\",\"eval_count\":0}\n".to_string(),
            ],
        },
    )
    .expect("parse tool call");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "request-1".to_string(),
            },
            ModelStreamEvent::ToolCall {
                call_id: "local-call-0".to_string(),
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
fn local_provider_stream_parser_accepts_split_jsonl_chunks() {
    let events = parse_local_jsonl(
        "request-1",
        &LocalHttpResponse {
            request_id: "request-1".to_string(),
            jsonl_chunks: vec![
                "{\"message\":{\"content\":\"he".to_string(),
                "llo\"},\"done\":false}\n{\"done\":".to_string(),
                "true,\"done_reason\":\"stop\",\"eval_count\":1}".to_string(),
            ],
        },
    )
    .expect("parse split jsonl chunks");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "request-1".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "hello".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 1,
            },
        ]
    );
}

#[test]
fn local_provider_errors_redact_response_bodies_and_classify_retryability() {
    let err = LocalError::provider_response(
        500,
        "{\"error\":\"local token sk-local-test-secret should not leak\"}",
    );
    let model_error: ModelError = err.into();
    let rendered = model_error.to_string();

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-local-test-secret"));
    assert!(matches!(
        model_error,
        ModelError::Provider {
            code,
            retryable: true,
            ..
        } if code == "model.local.provider_response"
    ));
    assert!(!LocalError::provider_response(400, "bad request").retryable());
}
