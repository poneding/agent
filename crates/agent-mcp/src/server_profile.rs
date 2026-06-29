use std::collections::BTreeSet;
use std::path::Path;

use agent_tool::{Tool, ToolRegistry};
use serde::{Deserialize, Serialize};

use crate::{
    McpError, McpPromptExport, McpPromptExportRegistry, McpResourceExportRegistry,
    McpToolExportRegistry,
};

const MAX_PROFILE_TEXT_CHARS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerExportProfile {
    pub name: String,
    pub tool_names: Vec<String>,
    pub prompt_names: Vec<String>,
    pub resource_uris: Vec<String>,
    #[serde(default)]
    pub resource_template_uri_templates: Vec<String>,
}

impl McpServerExportProfile {
    pub fn empty(name: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self {
            name: validate_profile_text("profile name", &name.into())?,
            tool_names: Vec::new(),
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            resource_template_uri_templates: Vec::new(),
        })
    }
}

pub struct McpServerProfileRuntime {
    profile: McpServerExportProfile,
    tools: ToolRegistry,
    tool_exports: McpToolExportRegistry,
    prompt_exports: McpPromptExportRegistry,
    resource_exports: McpResourceExportRegistry,
}

impl McpServerProfileRuntime {
    #[must_use]
    pub const fn profile(&self) -> &McpServerExportProfile {
        &self.profile
    }

    #[must_use]
    pub const fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    #[must_use]
    pub const fn tool_exports(&self) -> &McpToolExportRegistry {
        &self.tool_exports
    }

    #[must_use]
    pub const fn prompt_exports(&self) -> &McpPromptExportRegistry {
        &self.prompt_exports
    }

    #[must_use]
    pub const fn resource_exports(&self) -> &McpResourceExportRegistry {
        &self.resource_exports
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        McpServerExportProfile,
        ToolRegistry,
        McpToolExportRegistry,
        McpPromptExportRegistry,
        McpResourceExportRegistry,
    ) {
        (
            self.profile,
            self.tools,
            self.tool_exports,
            self.prompt_exports,
            self.resource_exports,
        )
    }
}

pub struct McpServerExportProfileBuilder {
    profile: McpServerExportProfile,
    tools: ToolRegistry,
    tool_exports: McpToolExportRegistry,
    prompt_exports: McpPromptExportRegistry,
    resource_exports: McpResourceExportRegistry,
    tool_names: BTreeSet<String>,
    prompt_names: BTreeSet<String>,
    resource_uris: BTreeSet<String>,
    resource_template_uri_templates: BTreeSet<String>,
}

impl McpServerExportProfileBuilder {
    pub fn new(name: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self {
            profile: McpServerExportProfile::empty(name)?,
            tools: ToolRegistry::default(),
            tool_exports: McpToolExportRegistry::default(),
            prompt_exports: McpPromptExportRegistry::default(),
            resource_exports: McpResourceExportRegistry::default(),
            tool_names: BTreeSet::new(),
            prompt_names: BTreeSet::new(),
            resource_uris: BTreeSet::new(),
            resource_template_uri_templates: BTreeSet::new(),
        })
    }

    pub fn export_tool<T>(&mut self, tool: T) -> Result<(), McpError>
    where
        T: Tool + 'static,
    {
        self.insert_tool(tool, true)
    }

    pub fn export_disabled_tool<T>(&mut self, tool: T) -> Result<(), McpError>
    where
        T: Tool + 'static,
    {
        self.insert_tool(tool, false)
    }

    pub fn export_prompt(&mut self, export: McpPromptExport) -> Result<(), McpError> {
        self.insert_prompt(export, true)
    }

    pub fn export_disabled_prompt(&mut self, export: McpPromptExport) -> Result<(), McpError> {
        self.insert_prompt(export, false)
    }

    pub fn export_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, true, false)
    }

    pub fn export_sensitive_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, true, true)
    }

    pub fn export_disabled_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, false, false)
    }

    pub fn export_disabled_sensitive_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, false, true)
    }

    pub fn export_file_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
        root: impl AsRef<Path>,
        path_template: impl Into<String>,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        self.insert_file_resource_template(
            uri_template,
            name,
            description,
            mime_type,
            root,
            path_template,
            true,
            false,
        )
    }

    pub fn export_sensitive_file_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
        root: impl AsRef<Path>,
        path_template: impl Into<String>,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        self.insert_file_resource_template(
            uri_template,
            name,
            description,
            mime_type,
            root,
            path_template,
            true,
            true,
        )
    }

    pub fn export_disabled_file_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
        root: impl AsRef<Path>,
        path_template: impl Into<String>,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        self.insert_file_resource_template(
            uri_template,
            name,
            description,
            mime_type,
            root,
            path_template,
            false,
            false,
        )
    }

    pub fn export_disabled_sensitive_file_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
        root: impl AsRef<Path>,
        path_template: impl Into<String>,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        self.insert_file_resource_template(
            uri_template,
            name,
            description,
            mime_type,
            root,
            path_template,
            false,
            true,
        )
    }

    #[must_use]
    pub fn build(mut self) -> McpServerProfileRuntime {
        self.profile.tool_names = self.tool_names.into_iter().collect();
        self.profile.prompt_names = self.prompt_names.into_iter().collect();
        self.profile.resource_uris = self.resource_uris.into_iter().collect();
        self.profile.resource_template_uri_templates =
            self.resource_template_uri_templates.into_iter().collect();
        McpServerProfileRuntime {
            profile: self.profile,
            tools: self.tools,
            tool_exports: self.tool_exports,
            prompt_exports: self.prompt_exports,
            resource_exports: self.resource_exports,
        }
    }

    fn insert_tool<T>(&mut self, tool: T, enabled: bool) -> Result<(), McpError>
    where
        T: Tool + 'static,
    {
        let definition = tool.definition();
        let name = validate_profile_text("tool name", definition.name.as_str())?;
        if self.tool_names.contains(&name)
            || self
                .tools
                .definitions()
                .iter()
                .any(|definition| definition.name.as_str() == name)
        {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server profile tool: {}",
                redact_text(&name)
            )));
        }
        crate::exported_tool_from_definition(definition.clone(), enabled)?;
        self.tools
            .register(tool)
            .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
        if enabled {
            self.tool_exports.export_tool(definition)?;
            self.tool_names.insert(name);
        } else {
            self.tool_exports.export_disabled_tool(definition)?;
        }
        Ok(())
    }

    fn insert_prompt(
        &mut self,
        mut export: McpPromptExport,
        enabled: bool,
    ) -> Result<(), McpError> {
        export.enabled = enabled;
        let name = validate_profile_text("prompt name", &export.descriptor().name)?;
        if enabled {
            self.prompt_exports.export_prompt(export)?;
            self.prompt_names.insert(name);
        } else {
            self.prompt_exports.export_disabled_prompt(export)?;
        }
        Ok(())
    }

    fn insert_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
        enabled: bool,
        sensitive: bool,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        let uri = validate_resource_uri(&uri.into())?;
        if self.resource_uris.contains(&uri)
            || self
                .resource_exports
                .list_resources()
                .iter()
                .any(|resource| resource.uri == uri)
        {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server profile resource: {}",
                redact_text(&uri)
            )));
        }
        if enabled {
            if sensitive {
                self.resource_exports
                    .export_sensitive_resource(&uri, name, mime_type, contents)?;
            } else {
                self.resource_exports
                    .export_resource(&uri, name, mime_type, contents)?;
            }
            self.resource_uris.insert(uri);
        } else {
            self.resource_exports
                .export_disabled_resource(&uri, name, mime_type, contents)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_file_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
        root: impl AsRef<Path>,
        path_template: impl Into<String>,
        enabled: bool,
        sensitive: bool,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        let uri_template = validate_resource_uri(&uri_template.into())?;
        match (enabled, sensitive) {
            (true, true) => self
                .resource_exports
                .export_sensitive_file_resource_template(
                    &uri_template,
                    name,
                    description,
                    mime_type,
                    root,
                    path_template,
                )?,
            (true, false) => self.resource_exports.export_file_resource_template(
                &uri_template,
                name,
                description,
                mime_type,
                root,
                path_template,
            )?,
            (false, true) => self
                .resource_exports
                .export_disabled_sensitive_file_resource_template(
                    &uri_template,
                    name,
                    description,
                    mime_type,
                    root,
                    path_template,
                )?,
            (false, false) => self
                .resource_exports
                .export_disabled_file_resource_template(
                    &uri_template,
                    name,
                    description,
                    mime_type,
                    root,
                    path_template,
                )?,
        }
        if enabled {
            self.resource_template_uri_templates.insert(uri_template);
        }
        Ok(())
    }
}

pub fn empty_mcp_server_export_profile(
    name: impl Into<String>,
) -> Result<McpServerProfileRuntime, McpError> {
    Ok(McpServerExportProfileBuilder::new(name)?.build())
}

fn validate_profile_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_PROFILE_TEXT_CHARS))
}

fn validate_resource_uri(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid resource uri: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_PROFILE_TEXT_CHARS))
}

fn redact_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    bound_text(value, MAX_PROFILE_TEXT_CHARS)
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
