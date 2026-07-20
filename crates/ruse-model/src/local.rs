use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::config::{ModelProviderConfig, ModelProviderKind};
use crate::provider::{ModelError, ModelProvider};
use crate::request::{ModelMessage, ModelRequest};
use crate::stream::{
    ModelEventStream, ModelStreamEvent, ModelStreamResult, model_events_to_stream,
};

const DEFAULT_LOCAL_BASE_URL: &str = "http://localhost:11434";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHttpRequest {
    pub url: String,
    pub body: Value,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHttpResponse {
    pub request_id: String,
    pub jsonl_chunks: Vec<String>,
}

#[async_trait]
pub trait LocalTransport: Send + Sync {
    async fn send(&self, request: LocalHttpRequest) -> Result<LocalHttpResponse, LocalError>;
}

#[derive(Clone, Debug)]
pub struct FixtureLocalTransport {
    response: Result<LocalHttpResponse, LocalError>,
}

impl FixtureLocalTransport {
    #[must_use]
    pub fn with_chunks(request_id: impl Into<String>, jsonl_chunks: Vec<String>) -> Self {
        Self {
            response: Ok(LocalHttpResponse {
                request_id: request_id.into(),
                jsonl_chunks,
            }),
        }
    }

    #[must_use]
    pub fn with_error(error: LocalError) -> Self {
        Self {
            response: Err(error),
        }
    }
}

#[async_trait]
impl LocalTransport for FixtureLocalTransport {
    async fn send(&self, _request: LocalHttpRequest) -> Result<LocalHttpResponse, LocalError> {
        self.response.clone()
    }
}

#[derive(Clone)]
pub struct ReqwestLocalTransport {
    client: reqwest::Client,
}

impl ReqwestLocalTransport {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestLocalTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ReqwestLocalTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReqwestLocalTransport")
            .field("client", &"<reqwest client>")
            .finish()
    }
}

#[async_trait]
impl LocalTransport for ReqwestLocalTransport {
    async fn send(&self, request: LocalHttpRequest) -> Result<LocalHttpResponse, LocalError> {
        let response = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .json(&request.body)
            .send()
            .await
            .map_err(|error| LocalError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;

        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("reqwest-response")
            .to_string();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LocalError::provider_response(status.as_u16(), &body));
        }

        let mut jsonl_chunks = Vec::new();
        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|error| LocalError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;
            jsonl_chunks.push(String::from_utf8_lossy(&chunk).to_string());
        }

        Ok(LocalHttpResponse {
            request_id,
            jsonl_chunks,
        })
    }
}

#[derive(Clone)]
pub struct LocalProvider {
    config: ModelProviderConfig,
    transport: Arc<dyn LocalTransport>,
}

impl LocalProvider {
    pub fn new(
        config: ModelProviderConfig,
        transport: Arc<dyn LocalTransport>,
    ) -> Result<Self, LocalError> {
        if config.kind != ModelProviderKind::Local {
            return Err(LocalError::InvalidProviderKind {
                provider_id: config.id,
            });
        }
        config.validate().map_err(|err| LocalError::InvalidConfig {
            message: err.to_string(),
        })?;
        Ok(Self { config, transport })
    }

    #[must_use]
    pub fn config(&self) -> &ModelProviderConfig {
        &self.config
    }
}

impl fmt::Debug for LocalProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ModelProvider for LocalProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        let http_request = LocalHttpRequest {
            url: local_chat_url(&self.config),
            body: build_local_request_body(&request),
            timeout_ms: self.config.endpoint.timeout_ms,
        };
        let response = self.transport.send(http_request).await?;
        let events = parse_local_jsonl(&response.request_id, &response)?;
        Ok(model_events_to_stream(events))
    }
}

#[must_use]
pub fn build_local_request_body(request: &ModelRequest) -> Value {
    let messages = request
        .messages
        .iter()
        .map(local_chat_message)
        .collect::<Vec<_>>();

    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
    });
    if !request.tools.is_empty() {
        body["tools"] = json!(
            request
                .tools
                .iter()
                .map(|tool| json!({
                    "type": "function",
                    "function": {
                        "name": tool.name.clone(),
                        "description": tool.description.clone(),
                        "parameters": tool.input_schema.clone(),
                    }
                }))
                .collect::<Vec<_>>()
        );
    }
    body
}

fn local_chat_message(message: &ModelMessage) -> Value {
    match message {
        ModelMessage::System { content } => json!({
            "role": "system",
            "content": content,
        }),
        ModelMessage::User { content } => json!({
            "role": "user",
            "content": content,
        }),
        ModelMessage::Assistant { content } => json!({
            "role": "assistant",
            "content": content,
        }),
        ModelMessage::AssistantToolCall {
            call_id,
            tool_name,
            input_json,
        } => json!({
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": call_id,
                "type": "function",
                "function": {
                    "name": tool_name,
                    "arguments": input_json.to_string(),
                }
            }],
        }),
        ModelMessage::ToolResult {
            call_id,
            tool_name,
            content,
        } => json!({
            "role": "tool",
            "tool_call_id": call_id,
            "name": tool_name,
            "content": content,
        }),
    }
}

#[must_use]
pub fn local_chat_url(config: &ModelProviderConfig) -> String {
    let base_url = config
        .endpoint
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_LOCAL_BASE_URL)
        .trim_end_matches('/');
    if base_url.ends_with("/api/chat") {
        base_url.to_string()
    } else {
        format!("{base_url}/api/chat")
    }
}

pub fn parse_local_jsonl(
    request_id: &str,
    response: &LocalHttpResponse,
) -> Result<ModelStreamResult, LocalError> {
    let mut events = vec![ModelStreamEvent::Started {
        request_id: request_id.to_string(),
    }];
    let mut tool_call_count = 0_u32;
    let mut buffer = String::new();

    for chunk in &response.jsonl_chunks {
        buffer.push_str(chunk);
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer.drain(..=line_end);
            handle_local_jsonl_line(&line, &mut events, &mut tool_call_count)?;
        }
    }
    handle_local_jsonl_line(buffer.trim(), &mut events, &mut tool_call_count)?;

    Ok(events)
}

fn handle_local_jsonl_line(
    line: &str,
    events: &mut ModelStreamResult,
    tool_call_count: &mut u32,
) -> Result<(), LocalError> {
    if line.is_empty() {
        return Ok(());
    }
    let value: Value =
        serde_json::from_str(line).map_err(|err| invalid_stream_event(err.to_string()))?;
    handle_local_chunk(&value, events, tool_call_count)
}

fn handle_local_chunk(
    value: &Value,
    events: &mut ModelStreamResult,
    tool_call_count: &mut u32,
) -> Result<(), LocalError> {
    if let Some(content) = value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .filter(|content| !content.is_empty())
    {
        events.push(ModelStreamEvent::Delta {
            text: content.to_string(),
        });
    }
    if let Some(tool_calls) = value
        .get("message")
        .and_then(|message| message.get("tool_calls"))
        .and_then(Value::as_array)
    {
        for tool_call in tool_calls {
            events.push(local_tool_call_event(tool_call, tool_call_count)?);
        }
    }
    if value.get("done").and_then(Value::as_bool).unwrap_or(false) {
        let finish_reason = value
            .get("done_reason")
            .and_then(Value::as_str)
            .unwrap_or("stop")
            .to_string();
        let output_token_count = value
            .get("eval_count")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .min(u64::from(u32::MAX)) as u32;
        events.push(ModelStreamEvent::Completed {
            finish_reason,
            output_token_count,
        });
    }
    Ok(())
}

fn local_tool_call_event(
    tool_call: &Value,
    tool_call_count: &mut u32,
) -> Result<ModelStreamEvent, LocalError> {
    let Some(function) = tool_call.get("function") else {
        return Err(invalid_stream_event("tool call function is missing"));
    };
    let Some(tool_name) = function.get("name").and_then(Value::as_str) else {
        return Err(invalid_stream_event("tool call function name is missing"));
    };
    let input_json = match function.get("arguments") {
        Some(Value::String(arguments)) => serde_json::from_str(arguments)
            .map_err(|err| invalid_stream_event(format!("invalid tool arguments JSON: {err}")))?,
        Some(arguments) => arguments.clone(),
        None => json!({}),
    };
    if !input_json.is_object() {
        return Err(invalid_stream_event(
            "tool call arguments must be an object",
        ));
    }
    let call_id = tool_call
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            let call_id = format!("local-call-{tool_call_count}");
            *tool_call_count = tool_call_count.saturating_add(1);
            call_id
        });
    Ok(ModelStreamEvent::ToolCall {
        call_id,
        tool_name: tool_name.to_string(),
        input_json,
    })
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LocalError {
    #[error("model provider {provider_id} is not local")]
    InvalidProviderKind { provider_id: String },
    #[error("invalid local provider config: {message}")]
    InvalidConfig { message: String },
    #[error("local transport failed: {message}")]
    Transport { message: String },
    #[error("local provider returned status {status}: {body_summary}")]
    ProviderResponse { status: u16, body_summary: String },
    #[error("invalid local stream event: {message}")]
    InvalidStreamEvent { message: String },
}

impl LocalError {
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
            Self::InvalidProviderKind { .. } => "model.local.invalid_provider_kind",
            Self::InvalidConfig { .. } => "model.local.invalid_config",
            Self::Transport { .. } => "model.local.transport",
            Self::ProviderResponse { .. } => "model.local.provider_response",
            Self::InvalidStreamEvent { .. } => "model.local.invalid_stream_event",
        }
    }

    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::Transport { .. } | Self::InvalidStreamEvent { .. } => true,
            Self::ProviderResponse { status, .. } => *status == 429 || *status >= 500,
            Self::InvalidProviderKind { .. } | Self::InvalidConfig { .. } => false,
        }
    }
}

impl From<LocalError> for ModelError {
    fn from(error: LocalError) -> Self {
        ModelError::Provider {
            code: error.code().to_string(),
            message: error.to_string(),
            retryable: error.retryable(),
        }
    }
}

fn invalid_stream_event(message: impl Into<String>) -> LocalError {
    LocalError::InvalidStreamEvent {
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
