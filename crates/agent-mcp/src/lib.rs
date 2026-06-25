#![forbid(unsafe_code)]
//! MCP protocol contracts and deterministic client fixtures for agent.

pub mod client;
pub mod config;
pub mod error;
pub mod policy;
pub mod tool;
pub mod tool_mapping;

pub use client::{
    FixtureMcpClient, McpClient, McpInitializeResult, McpServerCapabilities, McpServerHealth,
    McpServerStatus, check_mcp_server_health,
};
pub use config::{McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig};
pub use error::McpError;
pub use policy::{
    McpPermissionProfile, McpPolicyMapping, default_policy_mapping_for_mcp_tool,
    permission_request_for_mcp_mapping,
};
pub use tool::{McpToolDescriptor, McpToolListResult};
pub use tool_mapping::{
    McpLocalToolName, McpToolDefinition, local_tool_name_for_mcp_tool,
    mcp_descriptor_to_tool_definition,
};
