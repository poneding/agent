use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSourceKind {
    McpServer,
    Skill,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerHealthRecorded {
    pub server_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpToolsDiscovered {
    pub server_id: String,
    pub tool_count: u32,
    pub local_tool_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPolicyMappingRecorded {
    pub server_id: String,
    pub remote_tool_name: String,
    pub local_tool_name: String,
    pub action: String,
    pub resource_kind: String,
    pub resource_pattern: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillDiscoveredRecorded {
    pub skill_id: String,
    pub name: String,
    pub source_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillPromptContributionRecorded {
    pub skill_id: String,
    pub layer_id: String,
    pub priority: i32,
    pub token_estimate: u32,
    pub source_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginCapabilityValidated {
    pub plugin_name: String,
    pub capability_count: u32,
    pub tool_count: u32,
    pub hook_count: u32,
}
