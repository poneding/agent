use std::sync::Arc;

use agent_model::{
    FixtureGeminiTransport, GeminiError, GeminiHttpResponse, GeminiProvider, ModelEndpointConfig,
    ModelError, ModelMessage, ModelProvider, ModelProviderConfig, ModelProviderKind, ModelRequest,
    ModelStreamEvent, ModelToolDefinition, SecretRef, build_gemini_request_body,
    collect_model_event_stream, gemini_stream_generate_content_url, parse_gemini_sse,
};
use agent_types::{ConversationId, CorrelationId, ReasoningEffort, TurnId};
use serde_json::json;

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "gemini".to_string(),
        model: "gemini-1.5-pro".to_string(),
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
    SecretRef::new("env", "GEMINI_API_KEY").expect("valid secret ref")
}

fn provider_config(api_key: Option<SecretRef>) -> ModelProviderConfig {
    ModelProviderConfig {
        id: "gemini".to_string(),
        kind: ModelProviderKind::Gemini,
        default_model: "gemini-1.5-pro".to_string(),
        endpoint: ModelEndpointConfig::new(
            Some("https://generativelanguage.googleapis.com/v1beta"),
            30_000,
        )
        .expect("valid endpoint"),
        api_key,
        enabled: true,
    }
}

#[test]
fn gemini_request_builder_emits_generate_content_payload() {
    let body = build_gemini_request_body(&model_request());

    assert_eq!(
        body,
        json!({
            "systemInstruction": {
                "parts": [{ "text": "follow project instructions" }]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": "summarize this" }]
                }
            ],
            "generationConfig": {
                "maxOutputTokens": 1024
            }
        })
    );
}

#[test]
fn gemini_request_builder_maps_supported_reasoning_effort_dialects() {
    let mut request = model_request();
    request.reasoning_effort = Some(ReasoningEffort::Medium);

    request.model = "gemini-3-pro".to_string();
    let gemini_3 = build_gemini_request_body(&request);
    assert_eq!(
        gemini_3["generationConfig"]["thinkingConfig"],
        json!({ "thinkingLevel": "MEDIUM" })
    );

    request.model = "gemini-2.5-pro".to_string();
    let gemini_25 = build_gemini_request_body(&request);
    assert_eq!(
        gemini_25["generationConfig"]["thinkingConfig"],
        json!({ "thinkingBudget": 4096 })
    );

    request.model = "gemini-1.5-pro".to_string();
    let legacy = build_gemini_request_body(&request);
    assert!(legacy["generationConfig"].get("thinkingConfig").is_none());
}

#[test]
fn gemini_request_builder_emits_tools_payload() {
    let body = build_gemini_request_body(&tool_request());

    assert_eq!(body["contents"][1]["role"], "model");
    assert_eq!(
        body["contents"][2]["parts"][0]["functionResponse"]["name"],
        "workspace.read_file"
    );
    assert_eq!(
        body["tools"],
        json!([
            {
                "functionDeclarations": [
                    {
                        "name": "workspace.read_file",
                        "description": "Read a workspace file",
                        "parameters": {
                            "type": "object",
                            "required": ["path"],
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "additionalProperties": false
                        }
                    }
                ]
            }
        ])
    );
}

#[test]
fn gemini_url_builder_omits_api_key_from_url() {
    let url =
        gemini_stream_generate_content_url(&provider_config(Some(secret_ref())), "gemini-1.5-pro");

    assert_eq!(
        url,
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro:streamGenerateContent?alt=sse"
    );
    assert!(!url.contains("key="));
    assert!(!url.contains("GEMINI_API_KEY"));
}

#[test]
fn gemini_provider_requires_secret_ref() {
    let err = GeminiProvider::new(
        provider_config(None),
        Arc::new(FixtureGeminiTransport::with_chunks(
            "request-1",
            Vec::<String>::new(),
        )),
    )
    .expect_err("missing secret ref rejected");

    assert!(matches!(
        err,
        GeminiError::MissingSecretRef { provider_id } if provider_id == "gemini"
    ));
}

#[test]
fn gemini_provider_rejects_invalid_provider_kind() {
    let mut config = provider_config(Some(secret_ref()));
    config.kind = ModelProviderKind::Anthropic;

    let err = GeminiProvider::new(
        config,
        Arc::new(FixtureGeminiTransport::with_chunks(
            "request-1",
            Vec::<String>::new(),
        )),
    )
    .expect_err("invalid provider kind rejected");

    assert!(matches!(
        err,
        GeminiError::InvalidProviderKind { provider_id } if provider_id == "gemini"
    ));
}

#[tokio::test]
async fn gemini_stream_parser_maps_delta_and_completed_events() {
    let provider = GeminiProvider::new(
        provider_config(Some(secret_ref())),
        Arc::new(FixtureGeminiTransport::with_chunks(
            "request-1",
            vec![
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"hel\"}]}}]}\n\n"
                    .to_string(),
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"lo\"}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"candidatesTokenCount\":2}}\n\n"
                    .to_string(),
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
                finish_reason: "STOP".to_string(),
                output_token_count: 2,
            },
        ]
    );
}

#[test]
fn gemini_stream_parser_maps_function_calls() {
    let events = parse_gemini_sse(
        "request-1",
        &GeminiHttpResponse {
            request_id: "request-1".to_string(),
            sse_chunks: vec![
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"functionCall\":{\"name\":\"read_file\",\"args\":{\"path\":\"README.md\"}}}]}}]}\n\n"
                    .to_string(),
                "data: {\"candidates\":[{\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"candidatesTokenCount\":0}}\n\n"
                    .to_string(),
            ],
        },
    )
    .expect("parse function call");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "request-1".to_string(),
            },
            ModelStreamEvent::ToolCall {
                call_id: "gemini-call-0".to_string(),
                tool_name: "read_file".to_string(),
                input_json: json!({ "path": "README.md" }),
            },
            ModelStreamEvent::Completed {
                finish_reason: "STOP".to_string(),
                output_token_count: 0,
            },
        ]
    );
}

#[test]
fn gemini_errors_redact_response_bodies_and_classify_retryability() {
    let err = GeminiError::provider_response(
        403,
        "{\"error\":\"invalid key sk-gemini-test-secret api_key token should not leak\"}",
    );
    let model_error: ModelError = err.into();
    let rendered = model_error.to_string();

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-gemini-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(matches!(
        model_error,
        ModelError::Provider {
            code,
            retryable: false,
            ..
        } if code == "model.gemini.provider_response"
    ));
    assert!(GeminiError::provider_response(503, "unavailable").retryable());
}
