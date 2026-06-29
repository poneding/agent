use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    McpClientCapabilities, McpClientRootsCapability, McpClientSamplingCapability,
    McpCreateMessageRequestParams, McpCreateMessageResult, McpError, McpRootListResult,
    McpSamplingValidationPolicy, McpServerConfig, McpServerId, McpToolCallRequest,
    McpToolCallResult, McpToolDescriptor, McpToolListResult, validate_mcp_sampling_request,
};

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
pub trait McpClientRootProvider: Send + Sync {
    async fn list_roots(&self, server_id: &McpServerId) -> Result<McpRootListResult, McpError>;
}

#[async_trait]
pub trait McpSamplingProvider: Send + Sync {
    async fn create_message(
        &self,
        server_id: &McpServerId,
        request: McpCreateMessageRequestParams,
    ) -> Result<McpCreateMessageResult, McpError>;
}

#[async_trait]
pub trait McpClient: Send + Sync {
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError>;

    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError>;
}

#[async_trait]
pub trait McpGatewayClient: McpClient + McpClientRootProvider {
    async fn call_tool(&self, request: McpToolCallRequest) -> Result<McpToolCallResult, McpError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClient {
    servers: BTreeMap<McpServerId, McpInitializeResult>,
    tools: BTreeMap<McpServerId, McpToolListResult>,
    roots: BTreeMap<McpServerId, Result<McpRootListResult, McpError>>,
    sampling_results: BTreeMap<McpServerId, McpCreateMessageResult>,
    sampling_errors: BTreeMap<McpServerId, McpError>,
    tool_results: BTreeMap<(McpServerId, String), Result<McpToolCallResult, McpError>>,
    calls: Arc<Mutex<BTreeMap<(McpServerId, String), u32>>>,
    sampling_calls: Arc<Mutex<BTreeMap<McpServerId, u32>>>,
    client_capabilities: McpClientCapabilities,
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

    #[must_use]
    pub fn with_roots(mut self, server_id: McpServerId, roots: McpRootListResult) -> Self {
        self.roots.insert(server_id, Ok(roots));
        if self.client_capabilities.roots.is_none() {
            self.client_capabilities.roots = Some(McpClientRootsCapability::new(false));
        }
        self
    }

    #[must_use]
    pub fn with_root_error(mut self, server_id: McpServerId, error: McpError) -> Self {
        self.roots.insert(server_id, Err(error));
        if self.client_capabilities.roots.is_none() {
            self.client_capabilities.roots = Some(McpClientRootsCapability::new(false));
        }
        self
    }

    #[must_use]
    pub fn with_roots_list_changed(mut self, list_changed: bool) -> Self {
        self.client_capabilities.roots = Some(McpClientRootsCapability::new(list_changed));
        self
    }

    #[must_use]
    pub fn client_capabilities(&self) -> McpClientCapabilities {
        self.client_capabilities.clone()
    }

    #[must_use]
    pub fn with_tool_result(
        mut self,
        server_id: McpServerId,
        remote_tool_name: impl Into<String>,
        result: McpToolCallResult,
    ) -> Self {
        self.tool_results
            .insert((server_id, remote_tool_name.into()), Ok(result));
        self
    }

    #[must_use]
    pub fn with_tool_error(
        mut self,
        server_id: McpServerId,
        remote_tool_name: impl Into<String>,
        error: McpError,
    ) -> Self {
        self.tool_results
            .insert((server_id, remote_tool_name.into()), Err(error));
        self
    }

    #[must_use]
    pub fn with_sampling_result(
        mut self,
        server_id: McpServerId,
        result: McpCreateMessageResult,
    ) -> Self {
        self.sampling_errors.remove(&server_id);
        self.sampling_results.insert(server_id, result);
        self.client_capabilities.sampling = Some(McpClientSamplingCapability::basic());
        self
    }

    #[must_use]
    pub fn with_sampling_error(mut self, server_id: McpServerId, error: McpError) -> Self {
        self.sampling_results.remove(&server_id);
        self.sampling_errors.insert(server_id, error);
        self.client_capabilities.sampling = Some(McpClientSamplingCapability::basic());
        self
    }

    #[must_use]
    pub fn call_count(&self, server_id: &str, remote_tool_name: &str) -> u32 {
        let Ok(server_id) = McpServerId::new(server_id) else {
            return 0;
        };
        *self
            .calls
            .lock()
            .expect("fixture mcp calls")
            .get(&(server_id, remote_tool_name.to_string()))
            .unwrap_or(&0)
    }

    #[must_use]
    pub fn sampling_call_count(&self, server_id: &str) -> u32 {
        let Ok(server_id) = McpServerId::new(server_id) else {
            return 0;
        };
        *self
            .sampling_calls
            .lock()
            .expect("fixture mcp sampling calls")
            .get(&server_id)
            .unwrap_or(&0)
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

#[async_trait]
impl McpClientRootProvider for FixtureMcpClient {
    async fn list_roots(&self, server_id: &McpServerId) -> Result<McpRootListResult, McpError> {
        if !self.servers.contains_key(server_id) {
            return Err(McpError::ServerNotFound(server_id.to_string()));
        }
        Ok(self
            .roots
            .get(server_id)
            .cloned()
            .unwrap_or_else(|| Ok(McpRootListResult::empty()))?)
    }
}

#[async_trait]
impl McpSamplingProvider for FixtureMcpClient {
    async fn create_message(
        &self,
        server_id: &McpServerId,
        _request: McpCreateMessageRequestParams,
    ) -> Result<McpCreateMessageResult, McpError> {
        if !self.servers.contains_key(server_id) {
            return Err(McpError::ServerNotFound(server_id.to_string()));
        }

        *self
            .sampling_calls
            .lock()
            .expect("fixture mcp sampling calls")
            .entry(server_id.clone())
            .or_default() += 1;

        if let Some(error) = self.sampling_errors.get(server_id) {
            return Err(error.clone());
        }

        self.sampling_results
            .get(server_id)
            .cloned()
            .ok_or_else(|| {
                McpError::Client(format!(
                    "MCP sampling result not configured for server: {server_id}"
                ))
            })
    }
}

#[async_trait]
impl McpGatewayClient for FixtureMcpClient {
    async fn call_tool(&self, request: McpToolCallRequest) -> Result<McpToolCallResult, McpError> {
        let key = (request.server_id, request.remote_tool_name);
        *self
            .calls
            .lock()
            .expect("fixture mcp calls")
            .entry(key.clone())
            .or_default() += 1;
        self.tool_results
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Err(McpError::ServerNotFound(key.0.to_string())))
    }
}

pub async fn list_mcp_client_roots(
    client: &dyn McpClientRootProvider,
    config: &McpServerConfig,
) -> Result<McpRootListResult, McpError> {
    if !config.enabled {
        return Ok(McpRootListResult::empty());
    }
    client.list_roots(&config.id).await
}

pub async fn create_mcp_sampling_message(
    provider: &dyn McpSamplingProvider,
    config: &McpServerConfig,
    request: McpCreateMessageRequestParams,
) -> Result<McpCreateMessageResult, McpError> {
    if !config.enabled {
        return Err(McpError::InvalidConfig(
            "MCP server is disabled".to_string(),
        ));
    }

    let validation =
        validate_mcp_sampling_request(&request, &McpSamplingValidationPolicy::default());
    if !validation.accepted {
        let codes = validation
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::McpSamplingDiagnosticSeverity::Error)
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(McpError::InvalidServerRequest(format!(
            "sampling request rejected: {codes}"
        )));
    }

    provider.create_message(&config.id, request).await
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
