use agent_mcp::{
    McpCreateMessageRequestParams, McpSamplingContent, McpSamplingRole,
    McpSamplingValidationPolicy, validate_mcp_sampling_request,
};
use agent_model::ModelRequest;
use agent_types::{ConversationId, CorrelationId, TurnId};
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
