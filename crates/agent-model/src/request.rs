use agent_types::{ConversationId, CorrelationId, ReasoningEffort, TurnId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum ModelMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: String,
    },
    AssistantToolCall {
        call_id: String,
        tool_name: String,
        input_json: serde_json::Value,
    },
    ToolResult {
        call_id: String,
        tool_name: String,
        content: String,
    },
}

impl ModelMessage {
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
        }
    }

    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self::User {
            content: content.into(),
        }
    }

    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant {
            content: content.into(),
        }
    }

    #[must_use]
    pub fn assistant_tool_call(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        input_json: serde_json::Value,
    ) -> Self {
        Self::AssistantToolCall {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            input_json,
        }
    }

    #[must_use]
    pub fn tool_result(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::ToolResult {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            content: content.into(),
        }
    }

    #[must_use]
    pub fn text_content(&self) -> Option<&str> {
        match self {
            Self::System { content }
            | Self::User { content }
            | Self::Assistant { content }
            | Self::ToolResult { content, .. } => Some(content),
            Self::AssistantToolCall { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRequest {
    pub provider: String,
    pub model: String,
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
    pub messages: Vec<ModelMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ModelToolDefinition>,
}
