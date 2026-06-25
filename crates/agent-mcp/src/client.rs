use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{McpError, McpServerConfig, McpServerId, McpToolDescriptor, McpToolListResult};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerCapabilities {
    pub tools: bool,
    pub prompts: bool,
    pub resources: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpInitializeResult {
    pub server_id: McpServerId,
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: Option<String>,
    pub capabilities: McpServerCapabilities,
}

impl McpInitializeResult {
    pub fn new(
        server_id: McpServerId,
        protocol_version: impl Into<String>,
        server_name: impl Into<String>,
        server_version: Option<String>,
        capabilities: McpServerCapabilities,
    ) -> Self {
        Self {
            server_id,
            protocol_version: protocol_version.into(),
            server_name: server_name.into(),
            server_version,
            capabilities,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatus {
    Disabled,
    NotInitialized,
    Healthy,
    Unhealthy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerHealth {
    pub server_id: McpServerId,
    pub status: McpServerStatus,
    pub message: String,
}

#[async_trait]
pub trait McpClient: Send + Sync {
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError>;

    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClient {
    servers: BTreeMap<McpServerId, McpInitializeResult>,
    tools: BTreeMap<McpServerId, McpToolListResult>,
}

impl FixtureMcpClient {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_server(mut self, server_id: McpServerId, result: McpInitializeResult) -> Self {
        self.servers.insert(server_id, result);
        self
    }

    #[must_use]
    pub fn with_tools(mut self, server_id: McpServerId, tools: Vec<McpToolDescriptor>) -> Self {
        self.tools
            .insert(server_id.clone(), McpToolListResult::new(server_id, tools));
        self
    }
}

#[async_trait]
impl McpClient for FixtureMcpClient {
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError> {
        self.servers
            .get(&config.id)
            .cloned()
            .ok_or_else(|| McpError::ServerNotFound(config.id.to_string()))
    }

    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError> {
        self.tools
            .get(server_id)
            .cloned()
            .ok_or_else(|| McpError::ServerNotFound(server_id.to_string()))
    }
}

pub async fn check_mcp_server_health(
    client: &dyn McpClient,
    config: &McpServerConfig,
) -> McpServerHealth {
    if !config.enabled {
        return McpServerHealth {
            server_id: config.id.clone(),
            status: McpServerStatus::Disabled,
            message: "MCP server is disabled".to_string(),
        };
    }

    match client.initialize(config).await {
        Ok(init) => match client.list_tools(&config.id).await {
            Ok(tools) => McpServerHealth {
                server_id: config.id.clone(),
                status: McpServerStatus::Healthy,
                message: format!(
                    "initialized {} with {} tools",
                    init.server_name,
                    tools.tools.len()
                ),
            },
            Err(err) => McpServerHealth {
                server_id: config.id.clone(),
                status: McpServerStatus::Unhealthy,
                message: err.to_string(),
            },
        },
        Err(err) => McpServerHealth {
            server_id: config.id.clone(),
            status: McpServerStatus::Unhealthy,
            message: err.to_string(),
        },
    }
}
