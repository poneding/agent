use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;
use ruse_types::ReasoningEffort;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::config::{ModelProviderConfig, ModelProviderKind, SecretRef};
use crate::provider::{ModelError, ModelProvider};
use crate::request::{ModelMessage, ModelRequest};
use crate::secrets::SecretResolver;
use crate::stream::{
    ModelEventStream, ModelStreamEvent, ModelStreamResult, model_events_to_stream,
};

const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MAX_TOKENS: u32 = 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnthropicHttpRequest {
    pub url: String,
    pub secret_ref: SecretRef,
    pub body: Value,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnthropicHttpResponse {
    pub request_id: String,
    pub sse_chunks: Vec<String>,
}

#[async_trait]
pub trait AnthropicTransport: Send + Sync {
    async fn send(
        &self,
        request: AnthropicHttpRequest,
    ) -> Result<AnthropicHttpResponse, AnthropicError>;
}

#[derive(Clone, Debug)]
pub struct FixtureAnthropicTransport {
    response: Result<AnthropicHttpResponse, AnthropicError>,
}

impl FixtureAnthropicTransport {
    #[must_use]
    pub fn with_chunks(request_id: impl Into<String>, sse_chunks: Vec<String>) -> Self {
        Self {
            response: Ok(AnthropicHttpResponse {
                request_id: request_id.into(),
                sse_chunks,
            }),
        }
    }

    #[must_use]
    pub fn with_error(error: AnthropicError) -> Self {
        Self {
            response: Err(error),
        }
    }
}

#[async_trait]
impl AnthropicTransport for FixtureAnthropicTransport {
    async fn send(
        &self,
        _request: AnthropicHttpRequest,
    ) -> Result<AnthropicHttpResponse, AnthropicError> {
        self.response.clone()
    }
}

#[derive(Clone)]
pub struct ReqwestAnthropicTransport {
    client: reqwest::Client,
    secret_resolver: Arc<dyn SecretResolver>,
}

impl ReqwestAnthropicTransport {
    #[must_use]
    pub fn new(secret_resolver: Arc<dyn SecretResolver>) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret_resolver,
        }
    }

    #[must_use]
    pub fn with_client(secret_resolver: Arc<dyn SecretResolver>, client: reqwest::Client) -> Self {
        Self {
            client,
            secret_resolver,
        }
    }
}

impl fmt::Debug for ReqwestAnthropicTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReqwestAnthropicTransport")
            .field("client", &"<reqwest client>")
            .field("secret_resolver", &"<redacted>")
            .finish()
    }
}

#[async_trait]
impl AnthropicTransport for ReqwestAnthropicTransport {
    async fn send(
        &self,
        request: AnthropicHttpRequest,
    ) -> Result<AnthropicHttpResponse, AnthropicError> {
        let secret = self
            .secret_resolver
            .resolve(&request.secret_ref)
            .map_err(|error| AnthropicError::SecretResolution {
                message: error.to_string(),
            })?;
        let response = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .header("x-api-key", secret.expose_secret())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&request.body)
            .send()
            .await
            .map_err(|error| AnthropicError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;

        let status = response.status();
        let request_id = response
            .headers()
            .get("request-id")
            .or_else(|| response.headers().get("anthropic-request-id"))
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("reqwest-response")
            .to_string();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AnthropicError::provider_response(status.as_u16(), &body));
        }

        let mut sse_chunks = Vec::new();
        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|error| AnthropicError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;
            sse_chunks.push(String::from_utf8_lossy(&chunk).to_string());
        }

        Ok(AnthropicHttpResponse {
            request_id,
            sse_chunks,
        })
    }
}

#[derive(Clone)]
pub struct AnthropicProvider {
    config: ModelProviderConfig,
    secret_ref: SecretRef,
    transport: Arc<dyn AnthropicTransport>,
}

impl AnthropicProvider {
    pub fn new(
        config: ModelProviderConfig,
        transport: Arc<dyn AnthropicTransport>,
    ) -> Result<Self, AnthropicError> {
        if config.kind != ModelProviderKind::Anthropic {
            return Err(AnthropicError::InvalidProviderKind {
                provider_id: config.id,
            });
        }
        let Some(secret_ref) = config.api_key.clone() else {
            return Err(AnthropicError::MissingSecretRef {
                provider_id: config.id,
            });
        };
        config
            .validate()
            .map_err(|err| AnthropicError::InvalidConfig {
                message: err.to_string(),
            })?;
        Ok(Self {
            config,
            secret_ref,
            transport,
        })
    }

    #[must_use]
    pub fn config(&self) -> &ModelProviderConfig {
        &self.config
    }
}

impl fmt::Debug for AnthropicProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnthropicProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        let http_request = AnthropicHttpRequest {
            url: anthropic_messages_url(&self.config),
            secret_ref: self.secret_ref.clone(),
            body: build_anthropic_request_body(&request),
            timeout_ms: self.config.endpoint.timeout_ms,
        };
        let response = self.transport.send(http_request).await?;
        let events = parse_anthropic_sse(&response.request_id, &response)?;
        Ok(model_events_to_stream(events))
    }
}

#[must_use]
pub fn build_anthropic_request_body(request: &ModelRequest) -> Value {
    let system = request
        .messages
        .iter()
        .filter_map(|message| match message {
            ModelMessage::System { content } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let messages = request
        .messages
        .iter()
        .filter_map(anthropic_message)
        .collect::<Vec<_>>();

    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "max_tokens": DEFAULT_MAX_TOKENS,
        "stream": true,
    });
    if !system.is_empty() {
        body["system"] = json!(system);
    }
    if let Some(effort) = request.reasoning_effort {
        body["output_config"] = json!({ "effort": anthropic_effort(effort) });
    }
    if !request.tools.is_empty() {
        body["tools"] = json!(
            request
                .tools
                .iter()
                .map(|tool| json!({
                    "name": tool.name.clone(),
                    "description": tool.description.clone(),
                    "input_schema": tool.input_schema.clone(),
                }))
                .collect::<Vec<_>>()
        );
    }
    body
}

fn anthropic_effort(effort: ReasoningEffort) -> &'static str {
    match effort {
        ReasoningEffort::Minimal | ReasoningEffort::Low => "low",
        ReasoningEffort::Medium => "medium",
        ReasoningEffort::High => "high",
        ReasoningEffort::XHigh => "max",
    }
}

fn anthropic_message(message: &ModelMessage) -> Option<Value> {
    match message {
        ModelMessage::System { .. } => None,
        ModelMessage::User { content } => Some(json!({
            "role": "user",
            "content": [{
                "type": "text",
                "text": content,
            }],
        })),
        ModelMessage::Assistant { content } => Some(json!({
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": content,
            }],
        })),
        ModelMessage::AssistantToolCall {
            call_id,
            tool_name,
            input_json,
        } => Some(json!({
            "role": "assistant",
            "content": [{
                "type": "tool_use",
                "id": call_id,
                "name": tool_name,
                "input": input_json,
            }],
        })),
        ModelMessage::ToolResult {
            call_id, content, ..
        } => Some(json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": call_id,
                "content": content,
            }],
        })),
    }
}

pub fn parse_anthropic_sse(
    request_id: &str,
    response: &AnthropicHttpResponse,
) -> Result<ModelStreamResult, AnthropicError> {
    let mut events = vec![ModelStreamEvent::Started {
        request_id: request_id.to_string(),
    }];
    let mut tool_uses = BTreeMap::<u64, ToolUseState>::new();
    let mut finish_reason = None;
    let mut output_token_count = 0;

    for chunk in &response.sse_chunks {
        for frame in chunk.split("\n\n") {
            let frame = frame.trim();
            if frame.is_empty() {
                continue;
            }
            let Some((event_name, data)) = parse_sse_frame(frame) else {
                continue;
            };
            if data == "[DONE]" {
                continue;
            }
            let value: Value =
                serde_json::from_str(&data).map_err(|err| invalid_stream_event(err.to_string()))?;
            let event_type = event_name
                .as_deref()
                .or_else(|| value.get("type").and_then(Value::as_str));

            match event_type {
                Some("content_block_start") => handle_content_block_start(&value, &mut tool_uses),
                Some("content_block_delta") => {
                    handle_content_block_delta(&value, &mut tool_uses, &mut events);
                }
                Some("content_block_stop") => {
                    handle_content_block_stop(&value, &mut tool_uses, &mut events)?;
                }
                Some("message_delta") => {
                    if let Some(reason) = value
                        .get("delta")
                        .and_then(|delta| delta.get("stop_reason"))
                        .and_then(Value::as_str)
                    {
                        finish_reason = Some(reason.to_string());
                    }
                    if let Some(tokens) = value
                        .get("usage")
                        .and_then(|usage| usage.get("output_tokens"))
                        .and_then(Value::as_u64)
                    {
                        output_token_count = tokens.min(u64::from(u32::MAX)) as u32;
                    }
                }
                Some("message_stop") => events.push(ModelStreamEvent::Completed {
                    finish_reason: finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
                    output_token_count,
                }),
                _ => {}
            }
        }
    }

    Ok(events)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ToolUseState {
    call_id: String,
    tool_name: String,
    initial_input: Option<Value>,
    partial_json: String,
}

fn handle_content_block_start(value: &Value, tool_uses: &mut BTreeMap<u64, ToolUseState>) {
    let Some(index) = value.get("index").and_then(Value::as_u64) else {
        return;
    };
    let Some(content_block) = value.get("content_block") else {
        return;
    };
    if content_block.get("type").and_then(Value::as_str) != Some("tool_use") {
        return;
    }
    let Some(call_id) = content_block.get("id").and_then(Value::as_str) else {
        return;
    };
    let Some(tool_name) = content_block.get("name").and_then(Value::as_str) else {
        return;
    };
    let initial_input = content_block
        .get("input")
        .filter(|input| input.is_object())
        .cloned();
    tool_uses.insert(
        index,
        ToolUseState {
            call_id: call_id.to_string(),
            tool_name: tool_name.to_string(),
            initial_input,
            partial_json: String::new(),
        },
    );
}

fn handle_content_block_delta(
    value: &Value,
    tool_uses: &mut BTreeMap<u64, ToolUseState>,
    events: &mut ModelStreamResult,
) {
    let Some(delta) = value.get("delta") else {
        return;
    };
    match delta.get("type").and_then(Value::as_str) {
        Some("text_delta") => {
            if let Some(text) = delta.get("text").and_then(Value::as_str) {
                events.push(ModelStreamEvent::Delta {
                    text: text.to_string(),
                });
            }
        }
        Some("input_json_delta") => {
            let Some(index) = value.get("index").and_then(Value::as_u64) else {
                return;
            };
            let Some(partial_json) = delta.get("partial_json").and_then(Value::as_str) else {
                return;
            };
            if let Some(state) = tool_uses.get_mut(&index) {
                state.partial_json.push_str(partial_json);
            }
        }
        _ => {}
    }
}

fn handle_content_block_stop(
    value: &Value,
    tool_uses: &mut BTreeMap<u64, ToolUseState>,
    events: &mut ModelStreamResult,
) -> Result<(), AnthropicError> {
    let Some(index) = value.get("index").and_then(Value::as_u64) else {
        return Ok(());
    };
    let Some(state) = tool_uses.remove(&index) else {
        return Ok(());
    };
    let input_json = if state.partial_json.trim().is_empty() {
        state.initial_input.unwrap_or_else(|| json!({}))
    } else {
        serde_json::from_str(&state.partial_json)
            .map_err(|err| invalid_stream_event(format!("invalid tool input JSON: {err}")))?
    };
    if !input_json.is_object() {
        return Err(invalid_stream_event("tool input JSON must be an object"));
    }
    events.push(ModelStreamEvent::ToolCall {
        call_id: state.call_id,
        tool_name: state.tool_name,
        input_json,
    });
    Ok(())
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AnthropicError {
    #[error("Anthropic provider {provider_id} requires a secret reference")]
    MissingSecretRef { provider_id: String },
    #[error("model provider {provider_id} is not Anthropic")]
    InvalidProviderKind { provider_id: String },
    #[error("invalid Anthropic provider config: {message}")]
    InvalidConfig { message: String },
    #[error("Anthropic secret resolution failed: {message}")]
    SecretResolution { message: String },
    #[error("Anthropic transport failed: {message}")]
    Transport { message: String },
    #[error("Anthropic provider returned status {status}: {body_summary}")]
    ProviderResponse { status: u16, body_summary: String },
    #[error("invalid Anthropic stream event: {message}")]
    InvalidStreamEvent { message: String },
}

impl AnthropicError {
    #[must_use]
    pub fn provider_response(status: u16, body: &str) -> Self {
        Self::ProviderResponse {
            status,
            body_summary: redact_secret_like(body),
        }
    }

    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingSecretRef { .. } => "model.anthropic.missing_secret_ref",
            Self::InvalidProviderKind { .. } => "model.anthropic.invalid_provider_kind",
            Self::InvalidConfig { .. } => "model.anthropic.invalid_config",
            Self::SecretResolution { .. } => "model.anthropic.secret_resolution",
            Self::Transport { .. } => "model.anthropic.transport",
            Self::ProviderResponse { .. } => "model.anthropic.provider_response",
            Self::InvalidStreamEvent { .. } => "model.anthropic.invalid_stream_event",
        }
    }

    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::Transport { .. } | Self::InvalidStreamEvent { .. } => true,
            Self::ProviderResponse { status, .. } => *status == 429 || *status >= 500,
            Self::MissingSecretRef { .. }
            | Self::InvalidProviderKind { .. }
            | Self::InvalidConfig { .. }
            | Self::SecretResolution { .. } => false,
        }
    }
}

impl From<AnthropicError> for ModelError {
    fn from(error: AnthropicError) -> Self {
        ModelError::Provider {
            code: error.code().to_string(),
            message: error.to_string(),
            retryable: error.retryable(),
        }
    }
}

fn anthropic_messages_url(config: &ModelProviderConfig) -> String {
    let base_url = config
        .endpoint
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_ANTHROPIC_BASE_URL)
        .trim_end_matches('/');
    format!("{base_url}/messages")
}

fn parse_sse_frame(frame: &str) -> Option<(Option<String>, String)> {
    let mut event_name = None;
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        if let Some(raw_event) = line.strip_prefix("event:") {
            event_name = Some(raw_event.trim().to_string());
        } else if let Some(raw_data) = line.strip_prefix("data:") {
            data_lines.push(raw_data.trim().to_string());
        }
    }

    if data_lines.is_empty() {
        None
    } else {
        Some((event_name, data_lines.join("\n")))
    }
}

fn invalid_stream_event(message: impl Into<String>) -> AnthropicError {
    AnthropicError::InvalidStreamEvent {
        message: redact_secret_like(&message.into()),
    }
}

fn redact_secret_like(input: &str) -> String {
    let mut redacted = input.to_string();
    for token in input.split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))) {
        let lower = token.to_ascii_lowercase();
        if token.starts_with("sk-")
            || lower.contains("api_key")
            || lower.contains("token")
            || lower.contains("password")
            || lower.contains("secret")
        {
            redacted = redacted.replace(token, "[redacted]");
        }
    }
    if redacted.len() > 240 {
        redacted.truncate(240);
        redacted.push_str("...");
    }
    redacted
}

fn sanitize_url_in_message(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            if part.starts_with("http://") || part.starts_with("https://") {
                sanitize_url(part)
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_url(input: &str) -> String {
    let Some((scheme, rest)) = input.split_once("://") else {
        return input.to_string();
    };
    let rest = rest.split_once('@').map(|(_, tail)| tail).unwrap_or(rest);
    let (host_path, query_suffix) = match rest.split_once('?') {
        Some((host_path, _query)) => (host_path, "?<redacted>"),
        None => (rest, ""),
    };
    format!("{scheme}://{host_path}{query_suffix}")
}
