use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use futures_util::StreamExt;
use ruse_model::{
    FixtureOpenAiCompatibleTransport, ModelEndpointConfig, ModelError, ModelMessage, ModelProvider,
    ModelProviderConfig, ModelProviderKind, ModelRequest, ModelStreamEvent, ModelToolDefinition,
    OpenAiCompatibleError, OpenAiCompatibleHttpRequest, OpenAiCompatibleHttpResponse,
    OpenAiCompatibleHttpStreamResponse, OpenAiCompatibleProvider, OpenAiCompatibleTransport,
    SecretRef, build_openai_compatible_chat_request_body, build_openai_compatible_request_body,
    collect_model_event_stream, parse_openai_compatible_chat_sse, parse_openai_compatible_sse,
};
use ruse_types::{ConversationId, CorrelationId, ReasoningEffort, TurnId};
use serde_json::json;

#[derive(Debug)]
struct RecordingFallbackTransport {
    requests: Arc<Mutex<Vec<OpenAiCompatibleHttpRequest>>>,
}

#[async_trait]
impl OpenAiCompatibleTransport for RecordingFallbackTransport {
    async fn send(
        &self,
        request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError> {
        let mut requests = self.requests.lock().expect("requests");
        requests.push(request);
        if requests.len() == 1 {
            return Err(OpenAiCompatibleError::Transport {
                message: "connection closed".to_string(),
            });
        }
        Ok(OpenAiCompatibleHttpResponse {
            request_id: "chat-response-1".to_string(),
            sse_chunks: vec![
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\" there\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"completion_tokens\":2}}\n\n".to_string(),
            ],
        })
    }
}

#[derive(Debug)]
struct StreamingOnlyTransport {
    chunks: Vec<String>,
}

#[async_trait]
impl OpenAiCompatibleTransport for StreamingOnlyTransport {
    async fn send(
        &self,
        _request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError> {
        panic!("streaming provider should not buffer full response before returning")
    }

    async fn send_stream(
        &self,
        _request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpStreamResponse, OpenAiCompatibleError> {
        Ok(OpenAiCompatibleHttpStreamResponse::from_response(
            OpenAiCompatibleHttpResponse {
                request_id: "stream-response-1".to_string(),
                sse_chunks: self.chunks.clone(),
            },
        ))
    }
}

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "openai".to_string(),
        model: "gpt-4.1-mini".to_string(),
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
                { "role": "system", "content": "follow project instructions" },
                { "role": "user", "content": "summarize this" }
            ],
            "stream": true
        })
    );
}

#[test]
fn openai_compatible_request_builder_emits_chat_payload() {
    let body = build_openai_compatible_chat_request_body(&model_request());

    assert_eq!(
        body,
        json!({
            "model": "gpt-4.1-mini",
            "messages": [
                { "role": "system", "content": "follow project instructions" },
                { "role": "user", "content": "summarize this" }
            ],
            "stream": true
        })
    );
}

#[test]
fn openai_compatible_request_builders_emit_reasoning_effort() {
    let mut request = model_request();
    request.reasoning_effort = Some(ReasoningEffort::High);

    let responses = build_openai_compatible_request_body(&request);
    let chat = build_openai_compatible_chat_request_body(&request);

    assert_eq!(responses["reasoning"], json!({ "effort": "high" }));
    assert_eq!(chat["reasoning_effort"], json!("high"));
}

#[test]
fn openai_compatible_request_builder_emits_responses_tools_payload() {
    let body = build_openai_compatible_request_body(&tool_request());

    assert_eq!(body["input"][2]["type"], "function_call");
    assert_eq!(body["input"][3]["type"], "function_call_output");
    assert_eq!(
        body["tools"],
        json!([
            {
                "type": "function",
                "name": "workspace__read_file",
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
        ])
    );
}

#[test]
fn openai_compatible_request_builder_emits_chat_tools_payload() {
    let body = build_openai_compatible_chat_request_body(&tool_request());

    assert_eq!(body["messages"][2]["role"], "assistant");
    assert_eq!(body["messages"][3]["role"], "tool");
    assert_eq!(
        body["tools"],
        json!([
            {
                "type": "function",
                "function": {
                    "name": "workspace__read_file",
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
            }
        ])
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
fn openai_compatible_stream_parser_buffers_frames_split_across_chunks() {
    let events = parse_openai_compatible_sse(
        "response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "response-1".to_string(),
            sse_chunks: vec![
                "event: response.output_text.delta\ndata: {\"delta\":\"hel".to_string(),
                "lo\"}\n\n".to_string(),
                "event: response.completed\ndata: {\"finish_reason\":\"stop\",\"output_token_count\":2}".to_string(),
                "\n\n".to_string(),
            ],
        },
    )
    .expect("parse split stream frames");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "response-1".to_string()
            },
            ModelStreamEvent::Delta {
                text: "hello".to_string()
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 2,
            },
        ]
    );
}

#[test]
fn openai_compatible_chat_stream_parser_maps_delta_and_completed_events() {
    let events = parse_openai_compatible_chat_sse(
        "chat-response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "chat-response-1".to_string(),
            sse_chunks: vec![
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hel\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"lo\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"completion_tokens\":2}}\n\n".to_string(),
            ],
        },
    )
    .expect("parse chat stream");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "chat-response-1".to_string()
            },
            ModelStreamEvent::Delta {
                text: "hel".to_string()
            },
            ModelStreamEvent::Delta {
                text: "lo".to_string()
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 2,
            },
        ]
    );
}

#[test]
fn openai_compatible_responses_stream_parser_maps_reasoning_summary_delta() {
    let events = parse_openai_compatible_sse(
        "response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "response-1".to_string(),
            sse_chunks: vec![
                "event: response.reasoning_summary_text.delta\ndata: {\"delta\":\"checking context\"}\n\n".to_string(),
                "event: response.output_text.delta\ndata: {\"delta\":\"done\"}\n\n".to_string(),
            ],
        },
    )
    .expect("parse reasoning summary stream");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "response-1".to_string(),
            },
            ModelStreamEvent::ReasoningSummaryDelta {
                text: "checking context".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "done".to_string(),
            },
        ]
    );
}

#[test]
fn openai_compatible_chat_stream_parser_maps_reasoning_summary_delta() {
    let events = parse_openai_compatible_chat_sse(
        "chat-response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "chat-response-1".to_string(),
            sse_chunks: vec![
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"reasoning_content\":\"checking context\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"done\"},\"finish_reason\":null}]}\n\n".to_string(),
            ],
        },
    )
    .expect("parse chat reasoning summary stream");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "chat-response-1".to_string(),
            },
            ModelStreamEvent::ReasoningSummaryDelta {
                text: "checking context".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "done".to_string(),
            },
        ]
    );
}

#[test]
fn openai_compatible_chat_stream_parser_maps_streamed_tool_calls() {
    let response = OpenAiCompatibleHttpResponse {
        request_id: "chat-tool-1".to_string(),
        sse_chunks: vec![
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call-1\",\"type\":\"function\",\"function\":{\"name\":\"workspace.read_file\",\"arguments\":\"{\\\"path\\\":\"}}]},\"finish_reason\":null}]}\n\n".to_string(),
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"README.md\\\"}\"}}]},\"finish_reason\":null}]}\n\n".to_string(),
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}],\"usage\":{\"completion_tokens\":3}}\n\n".to_string(),
        ],
    };

    let events =
        parse_openai_compatible_chat_sse("chat-tool-1", &response).expect("chat tool parse");

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ModelStreamEvent::ToolCall {
                call_id,
                tool_name,
                input_json
            } if call_id == "call-1"
                && tool_name == "workspace.read_file"
                && input_json == &json!({ "path": "README.md" })
        )
    }));
}

#[test]
fn openai_compatible_chat_stream_parser_maps_safe_tool_names_back_to_internal_names() {
    let response = OpenAiCompatibleHttpResponse {
        request_id: "chat-tool-1".to_string(),
        sse_chunks: vec![
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call-1\",\"type\":\"function\",\"function\":{\"name\":\"workspace__read_file\",\"arguments\":\"{\\\"path\\\":\\\"README.md\\\"}\"}}]},\"finish_reason\":null}]}\n\n".to_string(),
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}],\"usage\":{\"completion_tokens\":3}}\n\n".to_string(),
        ],
    };

    let events =
        parse_openai_compatible_chat_sse("chat-tool-1", &response).expect("chat tool parse");

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ModelStreamEvent::ToolCall {
                call_id,
                tool_name,
                input_json
            } if call_id == "call-1"
                && tool_name == "workspace.read_file"
                && input_json == &json!({ "path": "README.md" })
        )
    }));
}

#[test]
fn openai_compatible_responses_stream_parser_maps_function_call_items() {
    let response = OpenAiCompatibleHttpResponse {
        request_id: "responses-tool-1".to_string(),
        sse_chunks: vec![
            "event: response.output_item.added\ndata: {\"item\":{\"id\":\"item-1\",\"type\":\"function_call\",\"call_id\":\"call-1\",\"name\":\"workspace.read_file\"}}\n\n".to_string(),
            "event: response.function_call_arguments.delta\ndata: {\"item_id\":\"item-1\",\"delta\":\"{\\\"path\\\":\\\"README.md\\\"}\"}\n\n".to_string(),
            "event: response.completed\ndata: {\"finish_reason\":\"tool_calls\",\"output_token_count\":0}\n\n".to_string(),
        ],
    };

    let events =
        parse_openai_compatible_sse("responses-tool-1", &response).expect("responses tool parse");

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ModelStreamEvent::ToolCall {
                call_id,
                tool_name,
                input_json
            } if call_id == "call-1"
                && tool_name == "workspace.read_file"
                && input_json == &json!({ "path": "README.md" })
        )
    }));
}

#[test]
fn openai_compatible_responses_stream_parser_maps_safe_tool_names_back_to_internal_names() {
    let response = OpenAiCompatibleHttpResponse {
        request_id: "responses-tool-1".to_string(),
        sse_chunks: vec![
            "event: response.output_item.added\ndata: {\"item\":{\"id\":\"item-1\",\"type\":\"function_call\",\"call_id\":\"call-1\",\"name\":\"workspace__read_file\"}}\n\n".to_string(),
            "event: response.function_call_arguments.delta\ndata: {\"item_id\":\"item-1\",\"delta\":\"{\\\"path\\\":\\\"README.md\\\"}\"}\n\n".to_string(),
            "event: response.completed\ndata: {\"finish_reason\":\"tool_calls\",\"output_token_count\":0}\n\n".to_string(),
        ],
    };

    let events =
        parse_openai_compatible_sse("responses-tool-1", &response).expect("responses tool parse");

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ModelStreamEvent::ToolCall {
                call_id,
                tool_name,
                input_json
            } if call_id == "call-1"
                && tool_name == "workspace.read_file"
                && input_json == &json!({ "path": "README.md" })
        )
    }));
}

#[test]
fn openai_compatible_stream_parser_ignores_incomplete_trailing_json_frame() {
    let events = parse_openai_compatible_sse(
        "response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "response-1".to_string(),
            sse_chunks: vec![
                "event: response.output_text.delta\ndata: {\"delta\":\"hello\"}\n\n".to_string(),
                "event: response.output_text.delta\ndata: {\"delta\":\"unfinished".to_string(),
            ],
        },
    )
    .expect("parse stream with truncated tail");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "response-1".to_string()
            },
            ModelStreamEvent::Delta {
                text: "hello".to_string()
            },
        ]
    );
}

#[test]
fn openai_compatible_chat_stream_parser_ignores_incomplete_trailing_json_frame() {
    let events = parse_openai_compatible_chat_sse(
        "chat-response-1",
        &OpenAiCompatibleHttpResponse {
            request_id: "chat-response-1".to_string(),
            sse_chunks: vec![
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n\n".to_string(),
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"unfinished".to_string(),
            ],
        },
    )
    .expect("parse chat stream with truncated tail");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "chat-response-1".to_string()
            },
            ModelStreamEvent::Delta {
                text: "hello".to_string()
            },
        ]
    );
}

#[tokio::test]
async fn openai_compatible_provider_falls_back_to_chat_completions() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let provider = OpenAiCompatibleProvider::new(
        ModelProviderConfig {
            id: "freemodel".to_string(),
            kind: ModelProviderKind::OpenAiCompatible,
            default_model: "gpt-5.5".to_string(),
            endpoint: ModelEndpointConfig::new(Some("https://api.freemodel.dev"), 30_000)
                .expect("valid endpoint"),
            api_key: Some(secret_ref()),
            enabled: true,
        },
        Arc::new(RecordingFallbackTransport {
            requests: Arc::clone(&requests),
        }),
    )
    .expect("valid provider");

    let stream = provider
        .stream(ModelRequest {
            provider: "freemodel".to_string(),
            model: "gpt-5.5".to_string(),
            conversation_id: ConversationId::new(),
            turn_id: TurnId::new(),
            correlation_id: CorrelationId::new(),
            messages: vec![ModelMessage::user("hello")],
            reasoning_effort: None,
            tools: Vec::new(),
        })
        .await
        .expect("fallback stream");
    let events = collect_model_event_stream(stream)
        .await
        .expect("collect fallback events");
    let recorded = requests.lock().expect("requests");

    assert_eq!(recorded[0].url, "https://api.freemodel.dev/responses");
    assert_eq!(
        recorded[1].url,
        "https://api.freemodel.dev/v1/chat/completions"
    );
    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "chat-response-1".to_string()
            },
            ModelStreamEvent::Delta {
                text: "hi".to_string()
            },
            ModelStreamEvent::Delta {
                text: " there".to_string()
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 2,
            },
        ]
    );
}

#[tokio::test]
async fn openai_compatible_provider_returns_before_full_stream_is_collected() {
    let provider = OpenAiCompatibleProvider::new(
        provider_config(Some(secret_ref())),
        Arc::new(StreamingOnlyTransport {
            chunks: vec![
                "event: response.output_text.delta\ndata: {\"delta\":\"hel\"}\n\n".to_string(),
                "event: response.output_text.delta\ndata: {\"delta\":\"lo\"}\n\n".to_string(),
            ],
        }),
    )
    .expect("valid provider");

    let mut stream = provider
        .stream(model_request())
        .await
        .expect("provider returns stream handle");

    assert_eq!(
        stream.next().await.transpose().expect("started event"),
        Some(ModelStreamEvent::Started {
            request_id: "stream-response-1".to_string(),
        })
    );
    assert_eq!(
        stream.next().await.transpose().expect("first delta"),
        Some(ModelStreamEvent::Delta {
            text: "hel".to_string(),
        })
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
