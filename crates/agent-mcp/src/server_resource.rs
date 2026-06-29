use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use agent_policy::{
    CapabilityAction, PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope,
};
use agent_types::{ActorRef, CorrelationId};
use serde::{Deserialize, Serialize};

use crate::McpError;

const DEFAULT_MAX_RESOURCE_CONTENT_BYTES: usize = 16 * 1024;
const MAX_RESOURCE_DIAGNOSTIC_CHARS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpResourceExport {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpResourceTemplateExport {
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpResourceReadResult {
    pub uri: String,
    pub contents: String,
    pub mime_type: Option<String>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct McpResourceRecord {
    export: McpResourceExport,
    contents: String,
    sensitive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct McpResourceTemplateRecord {
    export: McpResourceTemplateExport,
    path_root: Option<PathBuf>,
    path_template: Option<String>,
    sensitive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpResourceTemplateMatch {
    pub uri_template: String,
    pub uri: String,
    pub variables: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpResourceExportRegistry {
    resources: BTreeMap<String, McpResourceRecord>,
    resource_templates: BTreeMap<String, McpResourceTemplateRecord>,
    max_content_bytes: usize,
}

impl Default for McpResourceExportRegistry {
    fn default() -> Self {
        Self {
            resources: BTreeMap::new(),
            resource_templates: BTreeMap::new(),
            max_content_bytes: DEFAULT_MAX_RESOURCE_CONTENT_BYTES,
        }
    }
}

impl McpResourceExportRegistry {
    #[must_use]
    pub const fn with_max_content_bytes(mut self, max_content_bytes: usize) -> Self {
        self.max_content_bytes = max_content_bytes;
        self
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

    pub fn export_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        self.insert_resource_template(uri_template, name, description, mime_type, true)
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

    pub fn export_disabled_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        self.insert_resource_template(uri_template, name, description, mime_type, false)
    }

    #[must_use]
    pub fn list_resources(&self) -> Vec<McpResourceExport> {
        self.resources
            .values()
            .filter(|record| record.export.enabled)
            .map(|record| record.export.clone())
            .collect()
    }

    #[must_use]
    pub fn list_resource_templates(&self) -> Vec<McpResourceTemplateExport> {
        self.resource_templates
            .values()
            .filter(|record| record.export.enabled)
            .map(|record| record.export.clone())
            .collect()
    }

    #[must_use]
    pub fn match_resource_templates(&self, uri: &str) -> Vec<McpResourceTemplateMatch> {
        self.resource_templates
            .values()
            .filter(|record| record.export.enabled)
            .filter_map(|record| match_resource_template(&record.export.uri_template, uri))
            .collect()
    }

    pub fn read_resource(
        &self,
        policy: &dyn PolicyEngine,
        actor: ActorRef,
        correlation_id: CorrelationId,
        uri: &str,
    ) -> Result<McpResourceReadResult, McpError> {
        if let Some(record) = self
            .resources
            .get(uri)
            .filter(|record| record.export.enabled)
        {
            authorize_resource_read(policy, actor, correlation_id, &record.export.uri)?;
            let (contents, truncated) = truncate_content(&record.contents, self.max_content_bytes);
            return Ok(McpResourceReadResult {
                uri: record.export.uri.clone(),
                contents,
                mime_type: record.export.mime_type.clone(),
                truncated,
            });
        }

        self.read_template_resource(policy, actor, correlation_id, uri)
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
        let uri = validate_uri(&uri.into())?;
        if self.resources.contains_key(&uri) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP resource export: {}",
                redact_text(&uri)
            )));
        }
        let name = validate_non_empty("resource name", &name.into())?;
        let mime_type = mime_type.map(Into::into).map(|mime_type| {
            validate_non_empty("resource mime_type", &mime_type)
                .map(|value| bound_text(&value, MAX_RESOURCE_DIAGNOSTIC_CHARS))
        });
        let mime_type = match mime_type {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => return Err(error),
            None => None,
        };
        let contents = contents.into();
        if !sensitive && is_secret_like(&contents) {
            return Err(McpError::InvalidServerRequest(
                "resource content requires explicit sensitive export".to_string(),
            ));
        }

        self.resources.insert(
            uri.clone(),
            McpResourceRecord {
                export: McpResourceExport {
                    uri,
                    name,
                    mime_type,
                    enabled,
                },
                contents,
                sensitive,
            },
        );
        Ok(())
    }

    fn insert_resource_template<D, M>(
        &mut self,
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<D>,
        mime_type: Option<M>,
        enabled: bool,
    ) -> Result<(), McpError>
    where
        D: Into<String>,
        M: Into<String>,
    {
        let uri_template = validate_uri_template(&uri_template.into())?;
        if self.resource_templates.contains_key(&uri_template) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP resource template export: {}",
                redact_text(&uri_template)
            )));
        }
        let name = validate_non_empty("resource template name", &name.into())?;
        let description =
            validate_optional_public_text("resource template description", description)?;
        let mime_type = mime_type.map(Into::into).map(|mime_type| {
            validate_non_empty("resource template mime_type", &mime_type)
                .map(|value| bound_text(&value, MAX_RESOURCE_DIAGNOSTIC_CHARS))
        });
        let mime_type = match mime_type {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => return Err(error),
            None => None,
        };

        self.resource_templates.insert(
            uri_template.clone(),
            McpResourceTemplateRecord {
                export: McpResourceTemplateExport {
                    uri_template,
                    name,
                    description,
                    mime_type,
                    enabled,
                },
                path_root: None,
                path_template: None,
                sensitive: false,
            },
        );
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
        let path_root = canonicalize_template_root(root.as_ref())?;
        let path_template = validate_path_template(&path_template.into())?;
        let uri_template = validate_uri_template(&uri_template.into())?;
        if self.resource_templates.contains_key(&uri_template) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP resource template export: {}",
                redact_text(&uri_template)
            )));
        }
        let name = validate_non_empty("resource template name", &name.into())?;
        let description =
            validate_optional_public_text("resource template description", description)?;
        let mime_type = mime_type.map(Into::into).map(|mime_type| {
            validate_non_empty("resource template mime_type", &mime_type)
                .map(|value| bound_text(&value, MAX_RESOURCE_DIAGNOSTIC_CHARS))
        });
        let mime_type = match mime_type {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => return Err(error),
            None => None,
        };

        self.resource_templates.insert(
            uri_template.clone(),
            McpResourceTemplateRecord {
                export: McpResourceTemplateExport {
                    uri_template,
                    name,
                    description,
                    mime_type,
                    enabled,
                },
                path_root: Some(path_root),
                path_template: Some(path_template),
                sensitive,
            },
        );
        Ok(())
    }

    fn read_template_resource(
        &self,
        policy: &dyn PolicyEngine,
        actor: ActorRef,
        correlation_id: CorrelationId,
        uri: &str,
    ) -> Result<McpResourceReadResult, McpError> {
        let matches = self
            .resource_templates
            .values()
            .filter(|record| record.export.enabled && record.path_template.is_some())
            .filter_map(|record| {
                match_resource_template(&record.export.uri_template, uri)
                    .map(|matched| (record, matched))
            })
            .collect::<Vec<_>>();

        match matches.as_slice() {
            [] => Err(McpError::InvalidServerRequest(format!(
                "MCP resource export not found: {}",
                redact_text(uri)
            ))),
            [(_, _), _, ..] => Err(McpError::InvalidServerRequest(format!(
                "ambiguous MCP resource template for: {}",
                redact_text(uri)
            ))),
            [(record, matched)] => {
                let path = resolve_template_path(record, &matched.variables)?;
                authorize_resource_read(policy, actor, correlation_id, &matched.uri_template)?;
                let contents = std::fs::read_to_string(&path).map_err(|error| {
                    McpError::InvalidServerRequest(format!(
                        "failed to read MCP resource template path: {}",
                        redact_text(&error.to_string())
                    ))
                })?;
                if !record.sensitive && is_secret_like(&contents) {
                    return Err(McpError::InvalidServerRequest(
                        "resource content requires explicit sensitive export".to_string(),
                    ));
                }
                let (contents, truncated) = truncate_content(&contents, self.max_content_bytes);
                Ok(McpResourceReadResult {
                    uri: uri.to_string(),
                    contents,
                    mime_type: record.export.mime_type.clone(),
                    truncated,
                })
            }
        }
    }
}

fn authorize_resource_read(
    policy: &dyn PolicyEngine,
    actor: ActorRef,
    correlation_id: CorrelationId,
    uri: &str,
) -> Result<(), McpError> {
    let request = PermissionRequest::new(
        actor,
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri)
            .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?,
        format!("MCP server resource export requested: {uri}"),
        correlation_id,
    )
    .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
    let decision = policy
        .decide(&request)
        .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;

    match decision {
        PolicyDecision::Allow { .. } => {
            policy
                .issue_lease(
                    &request,
                    PolicyDecision::Allow {
                        reason: "MCP server resource export allowed".to_string(),
                    },
                )
                .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
            Ok(())
        }
        PolicyDecision::Deny { reason } | PolicyDecision::Ask { reason } => {
            Err(McpError::InvalidServerRequest(format!(
                "policy denied MCP resource export {}: {}",
                redact_text(uri),
                redact_text(&reason)
            )))
        }
    }
}

fn validate_uri(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP resource uri: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_RESOURCE_DIAGNOSTIC_CHARS))
}

fn validate_uri_template(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return invalid_resource_template(trimmed);
    }

    let mut cursor = 0;
    let mut variable_count = 0;
    while let Some(start_relative) = trimmed[cursor..].find('{') {
        let start = cursor + start_relative;
        if trimmed[cursor..start].contains('}') {
            return invalid_resource_template(trimmed);
        }
        let variable_start = start + 1;
        let Some(end_relative) = trimmed[variable_start..].find('}') else {
            return invalid_resource_template(trimmed);
        };
        let end = variable_start + end_relative;
        let variable = &trimmed[variable_start..end];
        if !valid_template_variable(variable) {
            return invalid_resource_template(trimmed);
        }
        variable_count += 1;
        cursor = end + 1;
    }
    if variable_count == 0 || trimmed[cursor..].contains('}') {
        return invalid_resource_template(trimmed);
    }

    Ok(bound_text(trimmed, MAX_RESOURCE_DIAGNOSTIC_CHARS))
}

fn match_resource_template(uri_template: &str, uri: &str) -> Option<McpResourceTemplateMatch> {
    let mut template_cursor = 0;
    let mut uri_cursor = 0;
    let mut variables = BTreeMap::new();

    while let Some(start_relative) = uri_template[template_cursor..].find('{') {
        let start = template_cursor + start_relative;
        let literal = &uri_template[template_cursor..start];
        if !uri[uri_cursor..].starts_with(literal) {
            return None;
        }
        uri_cursor += literal.len();

        let variable_start = start + 1;
        let end = variable_start + uri_template[variable_start..].find('}')?;
        let variable = &uri_template[variable_start..end];
        let after_variable = end + 1;
        let next_literal = next_template_literal(uri_template, after_variable);
        let value = if next_literal.is_empty() {
            let value = &uri[uri_cursor..];
            uri_cursor = uri.len();
            value
        } else {
            let next_position = uri[uri_cursor..].find(next_literal)?;
            let value = &uri[uri_cursor..uri_cursor + next_position];
            uri_cursor += next_position;
            value
        };
        if value.is_empty()
            || value.chars().count() > MAX_RESOURCE_DIAGNOSTIC_CHARS
            || is_secret_like(value)
        {
            return None;
        }
        variables.insert(variable.to_string(), value.to_string());
        template_cursor = after_variable;
    }

    let trailing = &uri_template[template_cursor..];
    if uri[uri_cursor..] != *trailing {
        return None;
    }

    Some(McpResourceTemplateMatch {
        uri_template: uri_template.to_string(),
        uri: uri.to_string(),
        variables,
    })
}

fn next_template_literal(uri_template: &str, cursor: usize) -> &str {
    match uri_template[cursor..].find('{') {
        Some(next_start) => &uri_template[cursor..cursor + next_start],
        None => &uri_template[cursor..],
    }
}

fn canonicalize_template_root(root: &Path) -> Result<PathBuf, McpError> {
    let canonical = root.canonicalize().map_err(|error| {
        McpError::InvalidServerRequest(format!(
            "invalid MCP resource template root: {}",
            redact_text(&error.to_string())
        ))
    })?;
    if !canonical.is_dir() {
        return Err(McpError::InvalidServerRequest(
            "invalid MCP resource template root: not a directory".to_string(),
        ));
    }
    Ok(canonical)
}

fn validate_path_template(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || Path::new(trimmed).is_absolute()
        || trimmed.chars().any(char::is_whitespace)
        || is_secret_like(trimmed)
    {
        return invalid_template_path(trimmed);
    }
    validate_braced_variables(trimmed)?;
    Ok(bound_text(trimmed, MAX_RESOURCE_DIAGNOSTIC_CHARS))
}

fn validate_braced_variables(value: &str) -> Result<(), McpError> {
    let mut cursor = 0;
    while let Some(start_relative) = value[cursor..].find('{') {
        let start = cursor + start_relative;
        if value[cursor..start].contains('}') {
            return invalid_template_path(value);
        }
        let variable_start = start + 1;
        let Some(end_relative) = value[variable_start..].find('}') else {
            return invalid_template_path(value);
        };
        let end = variable_start + end_relative;
        if !valid_template_variable(&value[variable_start..end]) {
            return invalid_template_path(value);
        }
        cursor = end + 1;
    }
    if value[cursor..].contains('}') {
        return invalid_template_path(value);
    }
    Ok(())
}

fn resolve_template_path(
    record: &McpResourceTemplateRecord,
    variables: &BTreeMap<String, String>,
) -> Result<PathBuf, McpError> {
    let root = record.path_root.as_ref().ok_or_else(|| {
        McpError::InvalidServerRequest("MCP resource template is not file-backed".to_string())
    })?;
    let path_template = record.path_template.as_ref().ok_or_else(|| {
        McpError::InvalidServerRequest("MCP resource template is not file-backed".to_string())
    })?;
    let rendered = render_template(path_template, variables)?;
    let relative = Path::new(&rendered);
    if rendered.is_empty() || relative.is_absolute() || unsafe_relative_path(relative) {
        return invalid_template_path(&rendered);
    }
    let resolved = root.join(relative).canonicalize().map_err(|error| {
        McpError::InvalidServerRequest(format!(
            "invalid MCP resource template path: {}",
            redact_text(&error.to_string())
        ))
    })?;
    if !resolved.starts_with(root) || !resolved.is_file() {
        return invalid_template_path(&rendered);
    }
    Ok(resolved)
}

fn render_template(
    template: &str,
    variables: &BTreeMap<String, String>,
) -> Result<String, McpError> {
    let mut rendered = String::new();
    let mut cursor = 0;
    while let Some(start_relative) = template[cursor..].find('{') {
        let start = cursor + start_relative;
        rendered.push_str(&template[cursor..start]);
        let variable_start = start + 1;
        let Some(end_relative) = template[variable_start..].find('}') else {
            return invalid_template_path(template);
        };
        let end = variable_start + end_relative;
        let variable = &template[variable_start..end];
        let Some(value) = variables.get(variable) else {
            return invalid_template_path(template);
        };
        rendered.push_str(value);
        cursor = end + 1;
    }
    rendered.push_str(&template[cursor..]);
    Ok(rendered)
}

fn unsafe_relative_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

fn invalid_template_path<T>(value: &str) -> Result<T, McpError> {
    Err(McpError::InvalidServerRequest(format!(
        "invalid MCP resource template path: {}",
        redact_text(value)
    )))
}

fn invalid_resource_template(value: &str) -> Result<String, McpError> {
    Err(McpError::InvalidServerRequest(format!(
        "invalid MCP resource template: {}",
        redact_text(value)
    )))
}

fn valid_template_variable(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn validate_non_empty(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_RESOURCE_DIAGNOSTIC_CHARS))
}

fn validate_optional_public_text<T>(
    field: &str,
    value: Option<T>,
) -> Result<Option<String>, McpError>
where
    T: Into<String>,
{
    value
        .map(|value| validate_non_empty(field, &value.into()))
        .transpose()
}

fn truncate_content(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_string(), false);
    }
    if max_bytes == 0 {
        return (String::new(), true);
    }
    let mut end = max_bytes.min(value.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_string(), true)
}

fn redact_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    bound_text(value, MAX_RESOURCE_DIAGNOSTIC_CHARS)
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
