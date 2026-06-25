#![forbid(unsafe_code)]
//! MCP protocol contracts and deterministic client fixtures for agent.

pub mod client;
pub mod config;
pub mod error;
pub mod tool;

pub use client::{
    FixtureMcpClient, McpClient, McpInitializeResult, McpServerCapabilities, McpServerHealth,
    McpServerStatus, check_mcp_server_health,
};
pub use config::{McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig};
pub use error::McpError;
pub use tool::{McpToolDescriptor, McpToolListResult};
