use agent_policy::{CapabilityAction, PermissionRequest, ResourceKind, ResourceScope};
use agent_tool::ToolName;
use agent_types::{ActorRef, CorrelationId};
use serde::{Deserialize, Serialize};

use crate::{McpError, McpServerId, McpToolDescriptor, local_tool_name_for_mcp_tool};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpPermissionProfile {
    ToolOnly,
    ReadOnly,
    Network,
    Execute,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPolicyMapping {
    pub server_id: McpServerId,
    pub remote_tool_name: String,
    pub local_tool_name: ToolName,
    pub profile: McpPermissionProfile,
    pub scope: ResourceScope,
    pub reason: String,
}

pub fn default_policy_mapping_for_mcp_tool(
    descriptor: &McpToolDescriptor,
) -> Result<McpPolicyMapping, McpError> {
    let local_tool_name = local_tool_name_for_mcp_tool(&descriptor.server_id, &descriptor.name)?;
    let remote_tool_name = descriptor.name.trim().to_string();
    if remote_tool_name.is_empty() {
        return Err(McpError::InvalidToolDescriptor(
            "remote tool name cannot be empty".to_string(),
        ));
    }

    let scope = ResourceScope::new(
        ResourceKind::Tool,
        format!("mcp:{}:{remote_tool_name}", descriptor.server_id.as_str()),
    )
    .map_err(|err| McpError::InvalidToolDescriptor(err.to_string()))?;

    Ok(McpPolicyMapping {
        server_id: descriptor.server_id.clone(),
        remote_tool_name,
        local_tool_name,
        profile: McpPermissionProfile::ToolOnly,
        scope,
        reason: format!(
            "MCP tool `{}` from server `{}` requires local permission",
            descriptor.name,
            descriptor.server_id.as_str()
        ),
    })
}

#[must_use]
pub fn permission_request_for_mcp_mapping(
    mapping: &McpPolicyMapping,
    actor: ActorRef,
) -> PermissionRequest {
    PermissionRequest::new(
        actor,
        action_for_profile(mapping.profile),
        mapping.scope.clone(),
        mapping.reason.clone(),
        CorrelationId::new(),
    )
    .expect("MCP policy mappings are validated before permission requests are built")
}

fn action_for_profile(profile: McpPermissionProfile) -> CapabilityAction {
    match profile {
        McpPermissionProfile::ReadOnly => CapabilityAction::Read,
        McpPermissionProfile::Network => CapabilityAction::Network,
        McpPermissionProfile::ToolOnly | McpPermissionProfile::Execute => CapabilityAction::Execute,
    }
}
