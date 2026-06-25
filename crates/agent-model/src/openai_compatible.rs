use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::config::{ModelProviderConfig, ModelProviderKind, SecretRef};
use crate::provider::{ModelError, ModelProvider};
use crate::request::ModelRequest;
use crate::stream::{ModelStreamEvent, ModelStreamResult};

const DEFAULT_OPENAI_COMPATIBLE_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiCompatibleHttpRequest {
    pub url: String,
    pub secret_ref: SecretRef,
    pub body: Value,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiCompatibleHttpResponse {
    pub request_id: String,
    pub sse_chunks: Vec<String>,
}

#[async_trait]
pub trait OpenAiCompatibleTransport: Send + Sync {
    async fn send(
        &self,
        request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError>;
}

#[derive(Clone, Debug)]
pub struct FixtureOpenAiCompatibleTransport {
    response: Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError>,
}

impl FixtureOpenAiCompatibleTransport {
    #[must_use]
    pub fn with_chunks(request_id: impl Into<String>, sse_chunks: Vec<String>) -> Self {
        Self {
            response: Ok(OpenAiCompatibleHttpResponse {
                request_id: request_id.into(),
                sse_chunks,
            }),
        }
    }

    #[must_use]
    pub fn with_error(error: OpenAiCompatibleError) -> Self {
        Self {
            response: Err(error),
        }
    }
}

#[async_trait]
impl OpenAiCompatibleTransport for FixtureOpenAiCompatibleTransport {
    async fn send(
        &self,
        _request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError> {
        self.response.clone()
    }
}

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    config: ModelProviderConfig,
    secret_ref: SecretRef,
    transport: Arc<dyn OpenAiCompatibleTransport>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        config: ModelProviderConfig,
        transport: Arc<dyn OpenAiCompatibleTransport>,
    ) -> Result<Self, OpenAiCompatibleError> {
        if config.kind != ModelProviderKind::OpenAiCompatible {
            return Err(OpenAiCompatibleError::InvalidProviderKind {
                provider_id: config.id,
            });
        }
        let Some(secret_ref) = config.api_key.clone() else {
            return Err(OpenAiCompatibleError::MissingSecretRef {
                provider_id: config.id,
            });
        };
        config
            .validate()
            .map_err(|err| OpenAiCompatibleError::InvalidConfig {
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

impl fmt::Debug for OpenAiCompatibleProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAiCompatibleProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ModelProvider for OpenAiCompatibleProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelStreamResult, ModelError> {
        let http_request = OpenAiCompatibleHttpRequest {
            url: openai_compatible_responses_url(&self.config),
            secret_ref: self.secret_ref.clone(),
            body: build_openai_compatible_request_body(&request),
            timeout_ms: self.config.endpoint.timeout_ms,
        };
        let response = self.transport.send(http_request).await?;
        parse_openai_compatible_sse(&response.request_id, &response).map_err(Into::into)
    }
}

#[must_use]
pub fn build_openai_compatible_request_body(request: &ModelRequest) -> Value {
    let input = request
        .messages
        .iter()
        .map(|message| {
            json!({
                "role": "user",
                "content": message,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "model": request.model,
        "input": input,
        "stream": true,
    })
}

pub fn parse_openai_compatible_sse(
    request_id: &str,
    response: &OpenAiCompatibleHttpResponse,
) -> Result<ModelStreamResult, OpenAiCompatibleError> {
    let mut events = vec![ModelStreamEvent::Started {
        request_id: request_id.to_string(),
    }];

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
            let value: Value = serde_json::from_str(&data).map_err(|err| {
                OpenAiCompatibleError::InvalidStreamEvent {
                    message: format!("invalid JSON stream data: {err}"),
                }
            })?;

            match event_name.as_deref() {
                Some("response.output_text.delta") => {
                    if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                        events.push(ModelStreamEvent::Delta {
                            text: delta.to_string(),
                        });
                    }
                }
                Some("response.completed") => {
                    events.push(ModelStreamEvent::Completed {
                        finish_reason: value
                            .get("finish_reason")
                            .and_then(Value::as_str)
                            .unwrap_or("stop")
                            .to_string(),
                        output_token_count: value
                            .get("output_token_count")
                            .and_then(Value::as_u64)
                            .unwrap_or(0)
                            .min(u64::from(u32::MAX))
                            as u32,
                    });
                }
                _ => {}
            }
        }
    }

    Ok(events)
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum OpenAiCompatibleError {
    #[error("OpenAI-compatible provider {provider_id} requires a secret reference")]
    MissingSecretRef { provider_id: String },
    #[error("model provider {provider_id} is not OpenAI-compatible")]
    InvalidProviderKind { provider_id: String },
    #[error("invalid OpenAI-compatible provider config: {message}")]
    InvalidConfig { message: String },
    #[error("OpenAI-compatible transport failed: {message}")]
    Transport { message: String },
    #[error("OpenAI-compatible provider returned status {status}: {body_summary}")]
    ProviderResponse { status: u16, body_summary: String },
    #[error("invalid OpenAI-compatible stream event: {message}")]
    InvalidStreamEvent { message: String },
}

impl OpenAiCompatibleError {
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
            Self::MissingSecretRef { .. } => "model.openai_compatible.missing_secret_ref",
            Self::InvalidProviderKind { .. } => "model.openai_compatible.invalid_provider_kind",
            Self::InvalidConfig { .. } => "model.openai_compatible.invalid_config",
            Self::Transport { .. } => "model.openai_compatible.transport",
            Self::ProviderResponse { .. } => "model.openai_compatible.provider_response",
            Self::InvalidStreamEvent { .. } => "model.openai_compatible.invalid_stream_event",
        }
    }

    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::Transport { .. } | Self::InvalidStreamEvent { .. } => true,
            Self::ProviderResponse { status, .. } => *status == 429 || *status >= 500,
            Self::MissingSecretRef { .. }
            | Self::InvalidProviderKind { .. }
            | Self::InvalidConfig { .. } => false,
        }
    }
}

impl From<OpenAiCompatibleError> for ModelError {
    fn from(error: OpenAiCompatibleError) -> Self {
        ModelError::Provider {
            code: error.code().to_string(),
            message: error.to_string(),
            retryable: error.retryable(),
        }
    }
}

fn openai_compatible_responses_url(config: &ModelProviderConfig) -> String {
    let base_url = config
        .endpoint
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_OPENAI_COMPATIBLE_BASE_URL)
        .trim_end_matches('/');
    format!("{base_url}/responses")
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

fn redact_secret_like(input: &str) -> String {
    let mut redacted = input.to_string();
    for token in input.split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))) {
        if token.starts_with("sk-") {
            redacted = redacted.replace(token, "[redacted]");
        }
    }
    if redacted.len() > 240 {
        redacted.truncate(240);
        redacted.push_str("...");
    }
    redacted
}
