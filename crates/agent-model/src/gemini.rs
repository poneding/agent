use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::config::{ModelProviderConfig, ModelProviderKind, SecretRef};
use crate::provider::{ModelError, ModelProvider};
use crate::request::ModelRequest;
use crate::secrets::SecretResolver;
use crate::stream::{
    ModelEventStream, ModelStreamEvent, ModelStreamResult, model_events_to_stream,
};

const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeminiHttpRequest {
    pub url: String,
    pub secret_ref: SecretRef,
    pub body: Value,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeminiHttpResponse {
    pub request_id: String,
    pub sse_chunks: Vec<String>,
}

#[async_trait]
pub trait GeminiTransport: Send + Sync {
    async fn send(&self, request: GeminiHttpRequest) -> Result<GeminiHttpResponse, GeminiError>;
}

#[derive(Clone, Debug)]
pub struct FixtureGeminiTransport {
    response: Result<GeminiHttpResponse, GeminiError>,
}

impl FixtureGeminiTransport {
    #[must_use]
    pub fn with_chunks(request_id: impl Into<String>, sse_chunks: Vec<String>) -> Self {
        Self {
            response: Ok(GeminiHttpResponse {
                request_id: request_id.into(),
                sse_chunks,
            }),
        }
    }

    #[must_use]
    pub fn with_error(error: GeminiError) -> Self {
        Self {
            response: Err(error),
        }
    }
}

#[async_trait]
impl GeminiTransport for FixtureGeminiTransport {
    async fn send(&self, _request: GeminiHttpRequest) -> Result<GeminiHttpResponse, GeminiError> {
        self.response.clone()
    }
}

#[derive(Clone)]
pub struct ReqwestGeminiTransport {
    client: reqwest::Client,
    secret_resolver: Arc<dyn SecretResolver>,
}

impl ReqwestGeminiTransport {
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

impl fmt::Debug for ReqwestGeminiTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReqwestGeminiTransport")
            .field("client", &"<reqwest client>")
            .field("secret_resolver", &"<redacted>")
            .finish()
    }
}

#[async_trait]
impl GeminiTransport for ReqwestGeminiTransport {
    async fn send(&self, request: GeminiHttpRequest) -> Result<GeminiHttpResponse, GeminiError> {
        let secret = self
            .secret_resolver
            .resolve(&request.secret_ref)
            .map_err(|error| GeminiError::SecretResolution {
                message: error.to_string(),
            })?;
        let response = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .header("x-goog-api-key", secret.expose_secret())
            .json(&request.body)
            .send()
            .await
            .map_err(|error| GeminiError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;

        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .or_else(|| response.headers().get("x-goog-request-id"))
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("reqwest-response")
            .to_string();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GeminiError::provider_response(status.as_u16(), &body));
        }

        let mut sse_chunks = Vec::new();
        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|error| GeminiError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;
            sse_chunks.push(String::from_utf8_lossy(&chunk).to_string());
        }

        Ok(GeminiHttpResponse {
            request_id,
            sse_chunks,
        })
    }
}

#[derive(Clone)]
pub struct GeminiProvider {
    config: ModelProviderConfig,
    secret_ref: SecretRef,
    transport: Arc<dyn GeminiTransport>,
}

impl GeminiProvider {
    pub fn new(
        config: ModelProviderConfig,
        transport: Arc<dyn GeminiTransport>,
    ) -> Result<Self, GeminiError> {
        if config.kind != ModelProviderKind::Gemini {
            return Err(GeminiError::InvalidProviderKind {
                provider_id: config.id,
            });
        }
        let Some(secret_ref) = config.api_key.clone() else {
            return Err(GeminiError::MissingSecretRef {
                provider_id: config.id,
            });
        };
        config
            .validate()
            .map_err(|err| GeminiError::InvalidConfig {
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

impl fmt::Debug for GeminiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeminiProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        let http_request = GeminiHttpRequest {
            url: gemini_stream_generate_content_url(&self.config, &request.model),
            secret_ref: self.secret_ref.clone(),
            body: build_gemini_request_body(&request),
            timeout_ms: self.config.endpoint.timeout_ms,
        };
        let response = self.transport.send(http_request).await?;
        let events = parse_gemini_sse(&response.request_id, &response)?;
        Ok(model_events_to_stream(events))
    }
}

#[must_use]
pub fn build_gemini_request_body(request: &ModelRequest) -> Value {
    let contents = request
        .messages
        .iter()
        .map(|message| {
            json!({
                "role": "user",
                "parts": [
                    {
                        "text": message,
                    }
                ],
            })
        })
        .collect::<Vec<_>>();

    json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": DEFAULT_MAX_OUTPUT_TOKENS,
        },
    })
}

#[must_use]
pub fn gemini_stream_generate_content_url(config: &ModelProviderConfig, model: &str) -> String {
    let base_url = config
        .endpoint
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_GEMINI_BASE_URL)
        .trim_end_matches('/');
    let model = model.trim().trim_start_matches("models/");
    format!("{base_url}/models/{model}:streamGenerateContent?alt=sse")
}

pub fn parse_gemini_sse(
    request_id: &str,
    response: &GeminiHttpResponse,
) -> Result<ModelStreamResult, GeminiError> {
    let mut events = vec![ModelStreamEvent::Started {
        request_id: request_id.to_string(),
    }];
    let mut tool_call_count = 0_u32;

    for chunk in &response.sse_chunks {
        for frame in chunk.split("\n\n") {
            let frame = frame.trim();
            if frame.is_empty() {
                continue;
            }
            let Some(data) = parse_sse_frame(frame) else {
                continue;
            };
            if data == "[DONE]" {
                continue;
            }
            let value: Value =
                serde_json::from_str(&data).map_err(|err| invalid_stream_event(err.to_string()))?;
            handle_gemini_chunk(&value, &mut events, &mut tool_call_count)?;
        }
    }

    Ok(events)
}

fn handle_gemini_chunk(
    value: &Value,
    events: &mut ModelStreamResult,
    tool_call_count: &mut u32,
) -> Result<(), GeminiError> {
    let output_token_count = value
        .get("usageMetadata")
        .and_then(|usage| usage.get("candidatesTokenCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .min(u64::from(u32::MAX)) as u32;
    let Some(candidates) = value.get("candidates").and_then(Value::as_array) else {
        return Ok(());
    };

    for candidate in candidates {
        if let Some(parts) = candidate
            .get("content")
            .and_then(|content| content.get("parts"))
            .and_then(Value::as_array)
        {
            for part in parts {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    events.push(ModelStreamEvent::Delta {
                        text: text.to_string(),
                    });
                }
                if let Some(function_call) = part.get("functionCall") {
                    events.push(gemini_tool_call_event(function_call, tool_call_count)?);
                }
            }
        }
        if let Some(finish_reason) = candidate.get("finishReason").and_then(Value::as_str) {
            events.push(ModelStreamEvent::Completed {
                finish_reason: finish_reason.to_string(),
                output_token_count,
            });
        }
    }
    Ok(())
}

fn gemini_tool_call_event(
    function_call: &Value,
    tool_call_count: &mut u32,
) -> Result<ModelStreamEvent, GeminiError> {
    let Some(tool_name) = function_call.get("name").and_then(Value::as_str) else {
        return Err(invalid_stream_event("function call name is missing"));
    };
    let input_json = function_call
        .get("args")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !input_json.is_object() {
        return Err(invalid_stream_event("function call args must be an object"));
    }
    let call_id = format!("gemini-call-{tool_call_count}");
    *tool_call_count = tool_call_count.saturating_add(1);
    Ok(ModelStreamEvent::ToolCall {
        call_id,
        tool_name: tool_name.to_string(),
        input_json,
    })
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum GeminiError {
    #[error("Gemini provider {provider_id} requires a secret reference")]
    MissingSecretRef { provider_id: String },
    #[error("model provider {provider_id} is not Gemini")]
    InvalidProviderKind { provider_id: String },
    #[error("invalid Gemini provider config: {message}")]
    InvalidConfig { message: String },
    #[error("Gemini secret resolution failed: {message}")]
    SecretResolution { message: String },
    #[error("Gemini transport failed: {message}")]
    Transport { message: String },
    #[error("Gemini provider returned status {status}: {body_summary}")]
    ProviderResponse { status: u16, body_summary: String },
    #[error("invalid Gemini stream event: {message}")]
    InvalidStreamEvent { message: String },
}

impl GeminiError {
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
            Self::MissingSecretRef { .. } => "model.gemini.missing_secret_ref",
            Self::InvalidProviderKind { .. } => "model.gemini.invalid_provider_kind",
            Self::InvalidConfig { .. } => "model.gemini.invalid_config",
            Self::SecretResolution { .. } => "model.gemini.secret_resolution",
            Self::Transport { .. } => "model.gemini.transport",
            Self::ProviderResponse { .. } => "model.gemini.provider_response",
            Self::InvalidStreamEvent { .. } => "model.gemini.invalid_stream_event",
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

impl From<GeminiError> for ModelError {
    fn from(error: GeminiError) -> Self {
        ModelError::Provider {
            code: error.code().to_string(),
            message: error.to_string(),
            retryable: error.retryable(),
        }
    }
}

fn parse_sse_frame(frame: &str) -> Option<String> {
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        if let Some(raw_data) = line.strip_prefix("data:") {
            data_lines.push(raw_data.trim().to_string());
        }
    }

    if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    }
}

fn invalid_stream_event(message: impl Into<String>) -> GeminiError {
    GeminiError::InvalidStreamEvent {
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
