use std::fmt;
use std::sync::Arc;

use agent_mcp::{
    McpCreateMessageRequestParams, McpCreateMessageResult, McpError, McpSamplingContent,
    McpSamplingProvider, McpSamplingRole, McpSamplingValidationPolicy, McpServerId,
    validate_mcp_sampling_request,
};
use agent_model::{ModelProvider, ModelRequest, ModelStreamEvent, collect_model_event_stream};
use agent_types::{ConversationId, CorrelationId, TurnId};
use async_trait::async_trait;
use thiserror::Error;

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
                ModelStreamEvent::Delta { text: delta } => text.push_str(&delta),
                ModelStreamEvent::Completed { finish_reason, .. } => {
                    stop_reason = Some(finish_reason);
                }
                ModelStreamEvent::ToolCall { .. } => {
                    return Err(McpError::InvalidServerRequest(
                        "sampling model stream rejected: tool_call".to_string(),
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
        messages.push(format!("system: {system_prompt}"));
    }

    for message in &request.messages {
        messages.push(format!(
            "{}: {}",
            sampling_role_label(message.role),
            text_content_for_model(&message.content.content)?
        ));
    }

    Ok(ModelRequest {
        provider,
        model,
        conversation_id: ids.conversation_id,
        turn_id: ids.turn_id,
        correlation_id: ids.correlation_id,
        messages,
    })
}

fn mcp_error_from_bridge_error(error: McpSamplingModelBridgeError) -> McpError {
    match error {
        McpSamplingModelBridgeError::InvalidRequest(code)
        | McpSamplingModelBridgeError::UnsupportedContent(code) => {
            McpError::InvalidServerRequest(format!("sampling request rejected: {code}"))
        }
        McpSamplingModelBridgeError::Model(code) => {
            McpError::Client(format!("sampling model bridge failed: {code}"))
        }
    }
}

fn redacted_model_error(_error: agent_model::ModelError) -> McpError {
    McpError::Client("sampling model provider failed".to_string())
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

fn sampling_role_label(role: McpSamplingRole) -> &'static str {
    match role {
        McpSamplingRole::User => "user",
        McpSamplingRole::Assistant => "assistant",
    }
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
