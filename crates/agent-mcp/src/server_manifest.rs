use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use agent_sandbox::WorkspaceSandbox;
use agent_tool::builtins::{GitDiffTool, GitStatusTool, ReadFileTool, SearchTextTool};
use serde::{Deserialize, Serialize};

use crate::{McpError, McpServerExportProfileBuilder, McpServerProfileRuntime};
use agent_tool::{Tool, ToolDefinition, ToolInput, ToolOutput};

pub const MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION: u32 = 1;

const MAX_MANIFEST_TEXT_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerExportManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub profiles: Vec<McpServerExportProfileConfig>,
}

impl McpServerExportManifest {
    pub fn validate(self) -> Result<Self, McpError> {
        if self.schema_version != MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION {
            return Err(McpError::InvalidServerRequest(format!(
                "unsupported MCP server export manifest schema_version: {}",
                self.schema_version
            )));
        }
        if self.profiles.is_empty() {
            return Err(McpError::InvalidServerRequest(
                "MCP server export manifest requires at least one profile".to_string(),
            ));
        }

        let mut profile_names = BTreeSet::new();
        for profile in &self.profiles {
            let name = validate_identifier_text("profile name", &profile.name)?;
            if !profile_names.insert(name.clone()) {
                return Err(McpError::InvalidServerRequest(format!(
                    "duplicate MCP server export profile: {}",
                    redact_text(&name)
                )));
            }
            validate_profile(profile)?;
        }

        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerExportProfileConfig {
    pub name: String,
    #[serde(default)]
    pub tools: Vec<McpServerToolExportConfig>,
    #[serde(default)]
    pub resources: Vec<McpServerResourceExportConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerToolExportConfig {
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerResourceExportConfig {
    pub uri: String,
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub sensitive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpServerBuiltinToolExportCatalogConfig {
    pub workspace_root: PathBuf,
}

impl McpServerBuiltinToolExportCatalogConfig {
    #[must_use]
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }
}

#[derive(Default)]
pub struct McpServerToolExportCatalog {
    tools: BTreeMap<String, Arc<dyn Tool>>,
}

impl McpServerToolExportCatalog {
    pub fn register<T>(&mut self, tool: T) -> Result<(), McpError>
    where
        T: Tool + 'static,
    {
        let definition = tool.definition();
        let name = validate_identifier_text("catalog tool name", definition.name.as_str())?;
        if self.tools.contains_key(&name) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server export catalog tool: {}",
                redact_text(&name)
            )));
        }
        self.tools.insert(name, Arc::new(tool));
        Ok(())
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    #[must_use]
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

pub fn build_mcp_server_builtin_tool_export_catalog(
    config: McpServerBuiltinToolExportCatalogConfig,
) -> Result<McpServerToolExportCatalog, McpError> {
    let sandbox = WorkspaceSandbox::new(&config.workspace_root).map_err(|error| {
        McpError::InvalidServerRequest(format!(
            "invalid MCP server builtin tool workspace root: {}",
            redact_text(&error.to_string())
        ))
    })?;

    let mut catalog = McpServerToolExportCatalog::default();
    catalog.register(ReadFileTool::new(sandbox.clone()))?;
    catalog.register(SearchTextTool::new(sandbox.clone()))?;
    catalog.register(GitStatusTool::new(sandbox.clone()))?;
    catalog.register(GitDiffTool::new(sandbox))?;
    Ok(catalog)
}

pub fn parse_mcp_server_export_manifest_json(
    input: &str,
) -> Result<McpServerExportManifest, McpError> {
    serde_json::from_str::<McpServerExportManifest>(input)
        .map_err(|_| {
            McpError::InvalidServerRequest("invalid MCP server export manifest JSON".to_string())
        })?
        .validate()
}

pub fn select_mcp_server_export_profile<'a>(
    manifest: &'a McpServerExportManifest,
    profile_name: Option<&str>,
) -> Result<&'a McpServerExportProfileConfig, McpError> {
    if let Some(profile_name) = profile_name {
        let wanted = validate_identifier_text("profile name", profile_name)?;
        return manifest
            .profiles
            .iter()
            .find(|profile| profile.name == wanted)
            .ok_or_else(|| {
                McpError::InvalidServerRequest(format!(
                    "MCP server export profile not found: {}",
                    redact_text(&wanted)
                ))
            });
    }

    if let Some(default) = manifest
        .profiles
        .iter()
        .find(|profile| profile.name == "default")
    {
        return Ok(default);
    }

    if manifest.profiles.len() == 1 {
        return Ok(&manifest.profiles[0]);
    }

    Err(McpError::InvalidServerRequest(
        "MCP server export profile name is required when manifest has multiple profiles"
            .to_string(),
    ))
}

pub fn build_mcp_server_resource_profile_from_manifest(
    manifest: &McpServerExportManifest,
    profile_name: Option<&str>,
    manifest_dir: impl AsRef<Path>,
) -> Result<McpServerProfileRuntime, McpError> {
    let profile = select_mcp_server_export_profile(manifest, profile_name)?;
    let mut builder = McpServerExportProfileBuilder::new(&profile.name)?;
    apply_manifest_resources_to_builder(profile, manifest_dir.as_ref(), &mut builder)?;

    Ok(builder.build())
}

pub fn build_mcp_server_tool_profile_from_manifest(
    manifest: &McpServerExportManifest,
    profile_name: Option<&str>,
    catalog: &McpServerToolExportCatalog,
) -> Result<McpServerProfileRuntime, McpError> {
    let profile = select_mcp_server_export_profile(manifest, profile_name)?;
    let mut builder = McpServerExportProfileBuilder::new(&profile.name)?;
    apply_manifest_tools_to_builder(profile, catalog, &mut builder)?;

    Ok(builder.build())
}

pub fn build_mcp_server_profile_from_manifest(
    manifest: &McpServerExportManifest,
    profile_name: Option<&str>,
    manifest_dir: impl AsRef<Path>,
    catalog: &McpServerToolExportCatalog,
) -> Result<McpServerProfileRuntime, McpError> {
    let profile = select_mcp_server_export_profile(manifest, profile_name)?;
    let mut builder = McpServerExportProfileBuilder::new(&profile.name)?;
    apply_manifest_resources_to_builder(profile, manifest_dir.as_ref(), &mut builder)?;
    apply_manifest_tools_to_builder(profile, catalog, &mut builder)?;

    Ok(builder.build())
}

fn apply_manifest_tools_to_builder(
    profile: &McpServerExportProfileConfig,
    catalog: &McpServerToolExportCatalog,
    builder: &mut McpServerExportProfileBuilder,
) -> Result<(), McpError> {
    for tool in &profile.tools {
        let catalog_tool = catalog.get(&tool.name).ok_or_else(|| {
            McpError::InvalidServerRequest(format!(
                "MCP server export catalog tool not found: {}; known tools: {}",
                redact_text(&tool.name),
                known_catalog_tool_names(catalog)
            ))
        })?;
        let exportable = SharedCatalogTool::new(catalog_tool);
        if tool.enabled {
            builder.export_tool(exportable)?;
        } else {
            builder.export_disabled_tool(exportable)?;
        }
    }

    Ok(())
}

fn known_catalog_tool_names(catalog: &McpServerToolExportCatalog) -> String {
    let names = catalog.tool_names();
    if names.is_empty() {
        return "none".to_string();
    }
    let joined = names
        .iter()
        .map(|name| redact_text(name))
        .collect::<Vec<_>>()
        .join(", ");
    bound_text(&joined, MAX_MANIFEST_TEXT_CHARS)
}

fn apply_manifest_resources_to_builder(
    profile: &McpServerExportProfileConfig,
    manifest_dir: &Path,
    builder: &mut McpServerExportProfileBuilder,
) -> Result<(), McpError> {
    for resource in &profile.resources {
        let path = resolve_manifest_resource_path(manifest_dir, &resource.path)?;
        let contents = fs::read_to_string(&path).map_err(|_| {
            McpError::InvalidServerRequest(format!(
                "failed to read MCP server export resource file: {}",
                redact_text(&resource.path)
            ))
        })?;

        match (resource.enabled, resource.sensitive) {
            (true, true) => builder.export_sensitive_resource(
                &resource.uri,
                &resource.name,
                resource.mime_type.clone(),
                contents,
            )?,
            (true, false) => builder.export_resource(
                &resource.uri,
                &resource.name,
                resource.mime_type.clone(),
                contents,
            )?,
            (false, true) => builder.export_disabled_sensitive_resource(
                &resource.uri,
                &resource.name,
                resource.mime_type.clone(),
                contents,
            )?,
            (false, false) => builder.export_disabled_resource(
                &resource.uri,
                &resource.name,
                resource.mime_type.clone(),
                contents,
            )?,
        }
    }

    Ok(())
}

struct SharedCatalogTool {
    tool: Arc<dyn Tool>,
}

impl SharedCatalogTool {
    fn new(tool: Arc<dyn Tool>) -> Self {
        Self { tool }
    }
}

impl Tool for SharedCatalogTool {
    fn definition(&self) -> ToolDefinition {
        self.tool.definition()
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, agent_tool::ToolError> {
        self.tool.execute(input)
    }
}

fn validate_profile(profile: &McpServerExportProfileConfig) -> Result<(), McpError> {
    let mut tool_names = BTreeSet::new();
    for tool in &profile.tools {
        let name = validate_identifier_text("tool name", &tool.name)?;
        if !tool_names.insert(name.clone()) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server export tool in profile {}: {}",
                redact_text(&profile.name),
                redact_text(&name)
            )));
        }
    }

    let mut resource_uris = BTreeSet::new();
    for resource in &profile.resources {
        let uri = validate_resource_uri(&resource.uri)?;
        if !resource_uris.insert(uri.clone()) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server export resource in profile {}: {}",
                redact_text(&profile.name),
                redact_text(&uri)
            )));
        }
        validate_public_text("resource name", &resource.name)?;
        validate_path_text("resource path", &resource.path)?;
        if let Some(mime_type) = &resource.mime_type {
            validate_public_text("resource mime_type", mime_type)?;
        }
    }

    Ok(())
}

fn validate_identifier_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn validate_public_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn validate_path_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn validate_resource_uri(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export resource uri: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn resolve_manifest_resource_path(
    manifest_dir: &Path,
    resource_path: &str,
) -> Result<PathBuf, McpError> {
    let path = Path::new(resource_path);
    if path.is_absolute() {
        return Err(McpError::InvalidServerRequest(format!(
            "MCP server export resource path must be relative to manifest directory: {}",
            redact_text(resource_path)
        )));
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(McpError::InvalidServerRequest(format!(
                    "MCP server export resource path must stay under manifest directory: {}",
                    redact_text(resource_path)
                )));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(McpError::InvalidServerRequest(
            "MCP server export resource path cannot be empty".to_string(),
        ));
    }

    Ok(manifest_dir.join(normalized))
}

fn default_enabled() -> bool {
    true
}

fn redact_text(value: &str) -> String {
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return REDACTED.to_string();
    }
    bound_text(value, MAX_MANIFEST_TEXT_CHARS)
}

fn bound_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}

fn is_secret_like(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace(['-', '.'], "_");
    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("private_key")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("sk_")
}
