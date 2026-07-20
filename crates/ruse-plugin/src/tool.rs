use std::sync::Arc;

use ruse_tool::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput, ToolRegistry};
use ruse_types::CorrelationId;

use crate::error::{PluginManifestError, PluginProcessError};
use crate::manifest::{PluginManifest, PluginName, PluginToolManifest};

#[derive(Clone, Debug, PartialEq)]
pub struct PluginToolInvocation {
    pub plugin_name: PluginName,
    pub tool_name: ToolName,
    pub call_id: String,
    pub input: serde_json::Value,
}

pub trait PluginToolHost: Send + Sync {
    fn invoke(&self, request: PluginToolInvocation) -> Result<ToolOutput, PluginProcessError>;
}

pub struct PluginTool {
    pub plugin_name: PluginName,
    pub definition: ToolDefinition,
    pub host: Arc<dyn PluginToolHost>,
}

impl PluginTool {
    pub fn from_manifest(
        manifest: &PluginManifest,
        tool: &PluginToolManifest,
        host: Arc<dyn PluginToolHost>,
    ) -> Result<Self, PluginManifestError> {
        Ok(Self {
            plugin_name: manifest.name.clone(),
            definition: ToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
            },
            host,
        })
    }
}

impl Tool for PluginTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let request = PluginToolInvocation {
            plugin_name: self.plugin_name.clone(),
            tool_name: self.definition.name.clone(),
            call_id: CorrelationId::new().to_string(),
            input: input.value,
        };

        self.host
            .invoke(request)
            .map_err(|error| ToolError::Execution(error.to_string()))
    }
}

pub struct PluginToolFactory {
    manifest: PluginManifest,
    host: Arc<dyn PluginToolHost>,
}

impl PluginToolFactory {
    #[must_use]
    pub fn new(manifest: PluginManifest, host: Arc<dyn PluginToolHost>) -> Self {
        Self { manifest, host }
    }

    pub fn build_tools(&self) -> Result<Vec<PluginTool>, PluginManifestError> {
        self.manifest
            .tools
            .iter()
            .map(|tool| PluginTool::from_manifest(&self.manifest, tool, self.host.clone()))
            .collect()
    }
}

pub fn register_plugin_tools(
    registry: &mut ToolRegistry,
    manifest: &PluginManifest,
    host: Arc<dyn PluginToolHost>,
) -> Result<Vec<ToolName>, PluginManifestError> {
    let mut registered = Vec::new();

    for tool in &manifest.tools {
        let plugin_tool = PluginTool::from_manifest(manifest, tool, host.clone())?;
        let name = plugin_tool.definition.name.clone();
        registry
            .register(plugin_tool)
            .map_err(|error| PluginManifestError::DuplicateContribution(error.to_string()))?;
        registered.push(name);
    }

    Ok(registered)
}
