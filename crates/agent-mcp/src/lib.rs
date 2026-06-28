#![forbid(unsafe_code)]
//! MCP protocol contracts and deterministic client fixtures for agent.

pub mod client;
pub mod config;
pub mod error;
pub mod gateway;
pub mod policy;
pub mod server;
pub mod server_tool;
pub mod tool;
pub mod tool_mapping;

pub use client::{
    FixtureMcpClient, McpClient, McpGatewayClient, McpInitializeResult, McpServerCapabilities,
    McpServerHealth, McpServerStatus, check_mcp_server_health,
};
pub use config::{McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig};
pub use error::McpError;
pub use gateway::{McpGateway, McpGatewayDiagnostic, McpGatewayReport, McpGatewayToolDescriptor};
pub use policy::{
    McpPermissionProfile, McpPolicyMapping, default_policy_mapping_for_mcp_tool,
    permission_request_for_mcp_mapping,
};
pub use server::{
    MCP_SERVER_PROTOCOL_VERSION, McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId,
    McpJsonRpcRequest, McpJsonRpcSuccess, McpServerInfo, McpServerMethod, McpServerState,
};
pub use server_tool::{McpToolExport, McpToolExportRegistry, exported_tool_from_definition};
pub use tool::{McpToolCallRequest, McpToolCallResult, McpToolDescriptor, McpToolListResult};
pub use tool_mapping::{
    McpLocalToolName, McpToolDefinition, local_tool_name_for_mcp_tool,
    mcp_descriptor_to_tool_definition,
};
