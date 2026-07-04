use agent_tool::ToolName;
use serde::{Deserialize, Serialize};

pub const PLUGIN_PROCESS_PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginProcessConfig {
    pub entrypoint: String,
    pub timeout_ms: u64,
    pub max_stdout_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginProcessRequest {
    pub protocol_version: u32,
    #[serde(rename = "type")]
    pub kind: PluginProcessRequestKind,
    pub call_id: String,
    pub tool_name: ToolName,
    pub input: serde_json::Value,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginProcessRequestKind {
    ToolCall,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginProcessResponse {
    pub protocol_version: u32,
    #[serde(rename = "type")]
    pub kind: PluginProcessResponseKind,
    pub call_id: String,
    #[serde(flatten)]
    pub result: PluginProcessResult,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginProcessResponseKind {
    ToolResult,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginProcessResult {
    Success {
        ok: bool,
        output: serde_json::Value,
    },
    Error {
        ok: bool,
        error_code: String,
        message: String,
        recoverable: bool,
    },
}
