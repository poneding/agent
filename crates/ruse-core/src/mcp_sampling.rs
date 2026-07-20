use std::fmt;
use std::sync::Arc;

use ruse_mcp::{
    McpCreateMessageRequestParams, McpCreateMessageResult, McpError, McpSamplingContent,
    McpSamplingProvider, McpSamplingRole, McpSamplingValidationPolicy, McpServerId,
    validate_mcp_sampling_request,
};
use ruse_model::{
    ModelError, ModelMessage, ModelProvider, ModelRequest, ModelStreamEvent,
    collect_model_event_stream,
};
use ruse_types::{ConversationId, CorrelationId, TurnId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_BRIDGE_DIAGNOSTIC_MESSAGE_CHARS: usize = 256;
const BRIDGE_REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpSamplingModelBridgeConfig {
    pub provider: String,
    pub model: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct McpSamplingModelBridgeIds {
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum McpSamplingModelBridgeError {
    #[error("invalid_request:{0}")]
    InvalidRequest(String),
    #[error("unsupported_content:{0}")]
    UnsupportedContent(String),
    #[error("model:{0}")]
    Model(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpSamplingBridgeDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Clone)]
pub struct McpModelSamplingProvider {
    pub config: McpSamplingModelBridgeConfig,
    ids: McpSamplingModelBridgeIds,
    model_provider: Arc<dyn ModelProvider>,
}

impl McpModelSamplingProvider {
    #[must_use]
    pub fn new(
        model_provider: Arc<dyn ModelProvider>,
        config: McpSamplingModelBridgeConfig,
        ids: McpSamplingModelBridgeIds,
    ) -> Self {
        Self {
            config,
            ids,
            model_provider,
        }
    }
}

impl fmt::Debug for McpModelSamplingProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("McpModelSamplingProvider")
            .field("config", &self.config)
            .field("ids", &self.ids)
            .field("model_provider", &"<dyn ModelProvider>")
            .finish()
    }
}

#[async_trait]
impl McpSamplingProvider for McpModelSamplingProvider {
    async fn create_message(
        &self,
        _server_id: &McpServerId,
        request: McpCreateMessageRequestParams,
    ) -> Result<McpCreateMessageResult, McpError> {
        let model_request = build_mcp_sampling_model_request(&request, &self.config, self.ids)
            .map_err(mcp_error_from_bridge_error)?;
        let stream = self
            .model_provider
            .stream(model_request)
            .await
            .map_err(redacted_model_error)?;
        let events = collect_model_event_stream(stream)
            .await
            .map_err(redacted_model_error)?;

        let mut text = String::new();
        let mut stop_reason = None;
        for event in events {
            match event {
                ModelStreamEvent::Started { .. } => {}
                ModelStreamEvent::ReasoningSummaryDelta { .. } => {}
                ModelStreamEvent::Delta { text: delta } => text.push_str(&delta),
                ModelStreamEvent::Completed { finish_reason, .. } => {
                    stop_reason = Some(finish_reason);
                }
                ModelStreamEvent::ToolCall { .. } => {
                    return Err(mcp_error_from_bridge_error(
                        McpSamplingModelBridgeError::UnsupportedContent(
                            "model_tool_call".to_string(),
                        ),
                    ));
                }
            }
        }

        McpCreateMessageResult::new(
            McpSamplingRole::Assistant,
            McpSamplingContent::text(text),
            self.config.model.as_str(),
            stop_reason,
        )
    }
}

pub fn build_mcp_sampling_model_request(
    request: &McpCreateMessageRequestParams,
    config: &McpSamplingModelBridgeConfig,
    ids: McpSamplingModelBridgeIds,
) -> Result<ModelRequest, McpSamplingModelBridgeError> {
    let provider = validate_local_model_field("provider", &config.provider)?;
    let model = validate_local_model_field("model", &config.model)?;

    let validation =
        validate_mcp_sampling_request(request, &McpSamplingValidationPolicy::default());
    if !validation.accepted {
        let codes = validation
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
            .join(",");
        return Err(McpSamplingModelBridgeError::InvalidRequest(codes));
    }

    let mut messages = Vec::new();
    if let Some(system_prompt) = &request.system_prompt {
        messages.push(ModelMessage::system(system_prompt));
    }

    for message in &request.messages {
        let content = text_content_for_model(&message.content.content)?;
        messages.push(match message.role {
            McpSamplingRole::User => ModelMessage::user(content),
            McpSamplingRole::Assistant => ModelMessage::assistant(content),
        });
    }

    Ok(ModelRequest {
        provider,
        model,
        conversation_id: ids.conversation_id,
        turn_id: ids.turn_id,
        correlation_id: ids.correlation_id,
        messages,
        reasoning_effort: None,
        tools: Vec::new(),
    })
}

#[must_use]
pub fn mcp_sampling_bridge_error_diagnostic(
    error: &McpSamplingModelBridgeError,
) -> McpSamplingBridgeDiagnostic {
    match error {
        McpSamplingModelBridgeError::InvalidRequest(detail) => bridge_diagnostic(
            "invalid_request",
            format!(
                "sampling request rejected: {}",
                sanitize_public_fragment(detail)
            ),
        ),
        McpSamplingModelBridgeError::UnsupportedContent(kind) => bridge_diagnostic(
            "unsupported_content",
            format!(
                "sampling content is unsupported: {}",
                sanitize_public_fragment(kind)
            ),
        ),
        McpSamplingModelBridgeError::Model(detail) => bridge_diagnostic(
            "model_bridge_failed",
            format!(
                "sampling model bridge failed: {}",
                sanitize_public_fragment(detail)
            ),
        ),
    }
}

#[must_use]
pub fn mcp_sampling_model_error_diagnostic(error: &ModelError) -> McpSamplingBridgeDiagnostic {
    match error {
        ModelError::Script(_) => bridge_diagnostic(
            "model_script_failed",
            "sampling model provider failed: script_error",
        ),
        ModelError::Provider { code, .. } => bridge_diagnostic(
            "model_provider_failed",
            format!(
                "sampling model provider failed: {}",
                sanitize_public_fragment(code)
            ),
        ),
    }
}

fn mcp_error_from_bridge_error(error: McpSamplingModelBridgeError) -> McpError {
    let diagnostic = mcp_sampling_bridge_error_diagnostic(&error);
    let public_message = format!("{}: {}", diagnostic.code, diagnostic.message);
    match error {
        McpSamplingModelBridgeError::InvalidRequest(_)
        | McpSamplingModelBridgeError::UnsupportedContent(_) => {
            McpError::InvalidServerRequest(public_message)
        }
        McpSamplingModelBridgeError::Model(_) => McpError::Client(public_message),
    }
}

fn redacted_model_error(error: ModelError) -> McpError {
    let diagnostic = mcp_sampling_model_error_diagnostic(&error);
    McpError::Client(format!("{}: {}", diagnostic.code, diagnostic.message))
}

fn validate_local_model_field(
    field: &str,
    value: &str,
) -> Result<String, McpSamplingModelBridgeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(McpSamplingModelBridgeError::InvalidRequest(format!(
            "invalid_config_{field}"
        )));
    }
    Ok(trimmed.to_string())
}

fn text_content_for_model(
    content: &[McpSamplingContent],
) -> Result<String, McpSamplingModelBridgeError> {
    let mut text = Vec::with_capacity(content.len());
    for block in content {
        match block {
            McpSamplingContent::Text { text: block_text } => text.push(block_text.as_str()),
            McpSamplingContent::Image { .. } => return unsupported_content("image"),
            McpSamplingContent::Audio { .. } => return unsupported_content("audio"),
            McpSamplingContent::ToolUse { .. } => return unsupported_content("tool_use"),
            McpSamplingContent::ToolResult { .. } => return unsupported_content("tool_result"),
        }
    }
    Ok(text.join("\n"))
}

fn unsupported_content<T>(kind: &str) -> Result<T, McpSamplingModelBridgeError> {
    Err(McpSamplingModelBridgeError::UnsupportedContent(
        kind.to_string(),
    ))
}

fn bridge_diagnostic(
    code: impl Into<String>,
    message: impl Into<String>,
) -> McpSamplingBridgeDiagnostic {
    McpSamplingBridgeDiagnostic {
        code: code.into(),
        message: bound_public_message(&message.into()),
    }
}

fn sanitize_public_fragment(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(trimmed) {
        return BRIDGE_REDACTED.to_string();
    }
    bound_public_message(trimmed)
}

fn bound_public_message(value: &str) -> String {
    value
        .chars()
        .take(MAX_BRIDGE_DIAGNOSTIC_MESSAGE_CHARS)
        .collect()
}

fn is_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("bearer ")
}
