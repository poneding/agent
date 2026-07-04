use std::sync::Arc;

use agent_model::{
    FixtureOpenAiCompatibleTransport, ModelEndpointConfig, ModelError, ModelProvider,
    ModelProviderConfig, ModelProviderKind, ModelRequest, ModelStreamEvent, OpenAiCompatibleError,
    OpenAiCompatibleHttpResponse, OpenAiCompatibleProvider, SecretRef,
    build_openai_compatible_request_body, collect_model_event_stream, parse_openai_compatible_sse,
};
use agent_types::{ConversationId, CorrelationId, TurnId};
use serde_json::json;

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "openai".to_string(),
        model: "gpt-4.1-mini".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec!["hello".to_string(), "summarize this".to_string()],
    }
}

fn secret_ref() -> SecretRef {
    SecretRef::new("env", "OPENAI_API_KEY").expect("valid secret ref")
}

fn provider_config(api_key: Option<SecretRef>) -> ModelProviderConfig {
    ModelProviderConfig {
        id: "openai".to_string(),
        kind: ModelProviderKind::OpenAiCompatible,
        default_model: "gpt-4.1-mini".to_string(),
        endpoint: ModelEndpointConfig::new(Some("https://api.openai.com/v1"), 30_000)
            .expect("valid endpoint"),
        api_key,
        enabled: true,
    }
}

#[test]
fn openai_compatible_request_builder_emits_responses_payload() {
    let body = build_openai_compatible_request_body(&model_request());

    assert_eq!(
        body,
        json!({
            "model": "gpt-4.1-mini",
            "input": [
                { "role": "user", "content": "hello" },
                { "role": "user", "content": "summarize this" }
            ],
            "stream": true
        })
    );
}

#[test]
fn openai_compatible_provider_requires_secret_ref() {
    let err = OpenAiCompatibleProvider::new(
        provider_config(None),
        Arc::new(FixtureOpenAiCompatibleTransport::with_chunks(
            "response-1",
            Vec::<String>::new(),
        )),
    )
    .expect_err("missing secret ref rejected");

    assert!(matches!(
        err,
        OpenAiCompatibleError::MissingSecretRef { provider_id } if provider_id == "openai"
    ));
}

#[tokio::test]
async fn openai_compatible_stream_parser_maps_delta_and_completed_events() {
    let provider = OpenAiCompatibleProvider::new(
        provider_config(Some(secret_ref())),
        Arc::new(FixtureOpenAiCompatibleTransport::with_chunks(
            "response-1",
            vec![
                "event: response.output_text.delta\ndata: {\"delta\":\"hel\"}\n\n".to_string(),
                "event: response.output_text.delta\ndata: {\"delta\":\"lo\"}\n\n".to_string(),
                "event: response.completed\ndata: {\"finish_reason\":\"stop\",\"output_token_count\":2}\n\n".to_string(),
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
                request_id: "response-1".to_string(),
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
fn openai_compatible_stream_parser_ignores_done_marker() {
    let events = parse_openai_compatible_sse(
        "response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "response-1".to_string(),
            sse_chunks: vec!["data: [DONE]\n\n".to_string()],
        },
    )
    .expect("parse done marker");

    assert_eq!(
        events,
        vec![ModelStreamEvent::Started {
            request_id: "response-1".to_string()
        }]
    );
}

#[test]
fn openai_compatible_errors_redact_response_bodies() {
    let err = OpenAiCompatibleError::provider_response(
        401,
        "{\"error\":\"invalid key sk-test-secret should not leak\"}",
    );
    let model_error: ModelError = err.into();
    let rendered = model_error.to_string();

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(matches!(
        model_error,
        ModelError::Provider {
            code,
            retryable: false,
            ..
        } if code == "model.openai_compatible.provider_response"
    ));
}
