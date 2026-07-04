use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ContextError;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLayerKind {
    System,
    Product,
    Project,
    Skill,
    Tool,
    Memory,
    Runtime,
    User,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptProfileId(String);

impl PromptProfileId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContextError> {
        let value = value.into();
        let value = value.trim();
        if value.is_empty() {
            return Err(ContextError::InvalidLayer(
                "profile id cannot be empty".to_string(),
            ));
        }
        Ok(Self(value.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PromptProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptLayerId(String);

impl PromptLayerId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContextError> {
        let value = value.into();
        let value = value.trim();
        if value.is_empty() {
            return Err(ContextError::InvalidLayer(
                "layer id cannot be empty".to_string(),
            ));
        }
        Ok(Self(value.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PromptLayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<&str> for PromptLayerId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<PromptLayerId> for &str {
    fn eq(&self, other: &PromptLayerId) -> bool {
        *self == other.as_str()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptBudget {
    pub max_tokens: u32,
    pub reserved_response_tokens: u32,
}

impl PromptBudget {
    pub fn new(max_tokens: u32, reserved_response_tokens: u32) -> Result<Self, ContextError> {
        if reserved_response_tokens >= max_tokens {
            return Err(ContextError::InvalidBudget(
                "reserved response tokens must be less than max tokens".to_string(),
            ));
        }
        Ok(Self {
            max_tokens,
            reserved_response_tokens,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptProfile {
    pub id: PromptProfileId,
    pub version: u32,
    pub display_name: String,
    pub required_layers: Vec<PromptLayerId>,
    pub default_budget: PromptBudget,
}

impl PromptProfile {
    pub fn new(
        id: PromptProfileId,
        version: u32,
        display_name: impl Into<String>,
        required_layers: Vec<PromptLayerId>,
        default_budget: PromptBudget,
    ) -> Result<Self, ContextError> {
        if version == 0 {
            return Err(ContextError::InvalidLayer(
                "profile version must be greater than zero".to_string(),
            ));
        }

        let display_name = display_name.into();
        let display_name = display_name.trim();
        if display_name.is_empty() {
            return Err(ContextError::InvalidLayer(
                "profile display name cannot be empty".to_string(),
            ));
        }

        let mut seen = HashSet::new();
        for layer_id in &required_layers {
            if !seen.insert(layer_id.clone()) {
                return Err(ContextError::InvalidLayer(format!(
                    "duplicate required layer id: {layer_id}"
                )));
            }
        }

        Ok(Self {
            id,
            version,
            display_name: display_name.to_string(),
            required_layers,
            default_budget,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PromptOverlayAction {
    Append,
    Replace { target_layer_id: PromptLayerId },
    Disable { target_layer_id: PromptLayerId },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptOverlay {
    pub id: String,
    pub priority: i32,
    pub action: PromptOverlayAction,
    pub layer: Option<PromptLayer>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptAssemblyRequest {
    pub profile: PromptProfile,
    pub layers: Vec<PromptLayer>,
    pub overlays: Vec<PromptOverlay>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptAssemblyOutcome {
    pub bundle: PromptBundle,
    pub report: PromptAssemblyReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptAssemblyReport {
    pub profile_id: PromptProfileId,
    pub profile_version: u32,
    pub included_layer_ids: Vec<PromptLayerId>,
    pub omitted_layer_ids: Vec<PromptLayerId>,
    pub estimated_tokens: u32,
    pub diagnostics: Vec<PromptDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptLayerHash {
    pub layer_id: PromptLayerId,
    pub source_hash: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptSnapshot {
    pub profile_id: PromptProfileId,
    pub profile_version: u32,
    pub layer_hashes: Vec<PromptLayerHash>,
    pub bundle_hash: String,
    pub diagnostics: Vec<PromptDiagnostic>,
    pub message_count: u32,
    pub estimated_tokens: u32,
}

impl PromptSnapshot {
    #[must_use]
    pub fn from_assembly(bundle: &PromptBundle, report: &PromptAssemblyReport) -> Self {
        let layer_hashes: Vec<_> = bundle
            .layers
            .iter()
            .map(|layer| PromptLayerHash {
                layer_id: layer.id.clone(),
                source_hash: stable_json_hash(&layer.source),
                content_hash: stable_str_hash(&layer.content),
            })
            .collect();
        let bundle_hash = stable_json_hash(&serde_json::json!({
            "profile_id": report.profile_id.as_str(),
            "profile_version": report.profile_version,
            "messages": bundle.messages,
            "layer_hashes": layer_hashes,
            "estimated_tokens": bundle.estimated_tokens,
        }));

        Self {
            profile_id: report.profile_id.clone(),
            profile_version: report.profile_version,
            layer_hashes,
            bundle_hash,
            diagnostics: report.diagnostics.clone(),
            message_count: bundle.messages.len() as u32,
            estimated_tokens: bundle.estimated_tokens,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptDiagnostic {
    pub severity: PromptDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub layer_id: Option<PromptLayerId>,
}

impl PromptDiagnostic {
    fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        layer_id: Option<PromptLayerId>,
    ) -> Self {
        Self {
            severity: PromptDiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            layer_id,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PromptRuntime;

impl PromptRuntime {
    pub fn assemble(
        &self,
        request: PromptAssemblyRequest,
    ) -> Result<PromptAssemblyOutcome, ContextError> {
        let mut diagnostics = Vec::new();
        let mut layers = request.layers;

        for layer in &layers {
            validate_layer(layer)?;
        }

        layers = apply_overlays(layers, request.overlays, &request.profile, &mut diagnostics)?;
        layers = deduplicate_layers(layers, &mut diagnostics)?;
        ensure_required_layers(&request.profile, &layers)?;
        let (layers, omitted_layer_ids) =
            apply_prompt_budget(&request.profile, layers, &mut diagnostics)?;
        let bundle = assemble_prompt_bundle(layers)?;
        let included_layer_ids = bundle.layers.iter().map(|layer| layer.id.clone()).collect();

        Ok(PromptAssemblyOutcome {
            report: PromptAssemblyReport {
                profile_id: request.profile.id,
                profile_version: request.profile.version,
                included_layer_ids,
                omitted_layer_ids,
                estimated_tokens: bundle.estimated_tokens,
                diagnostics,
            },
            bundle,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PromptLayerSource {
    BuiltIn,
    RuntimeConfig { source: String },
    ProjectFile { path: String, content_hash: String },
    UserPreference { key: String },
    Skill { skill_id: String, source: String },
    Tool { tool_name: String },
    Memory { memory_id: String },
    TeamRole { role_id: String },
    SessionOverlay { command_id: String },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLayerVisibility {
    Hidden,
    SummaryOnly,
    Full,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLayerSensitivity {
    Public,
    Workspace,
    UserPrivate,
    SecretLike,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLayerPolicy {
    Required,
    Optional,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptLayer {
    pub id: PromptLayerId,
    pub kind: PromptLayerKind,
    pub priority: i32,
    pub content: String,
    pub token_estimate: u32,
    pub source: PromptLayerSource,
    pub visibility: PromptLayerVisibility,
    pub sensitivity: PromptLayerSensitivity,
    pub policy: PromptLayerPolicy,
    pub version: u32,
}

impl PromptLayer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        kind: PromptLayerKind,
        priority: i32,
        content: impl Into<String>,
        token_estimate: u32,
        source: PromptLayerSource,
        visibility: PromptLayerVisibility,
        sensitivity: PromptLayerSensitivity,
        policy: PromptLayerPolicy,
        version: u32,
    ) -> Result<Self, ContextError> {
        let layer = Self {
            id: PromptLayerId::new(id)?,
            kind,
            priority,
            content: content.into(),
            token_estimate,
            source,
            visibility,
            sensitivity,
            policy,
            version,
        };
        validate_layer(&layer)?;
        Ok(layer)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptBundle {
    pub layers: Vec<PromptLayer>,
    pub messages: Vec<PromptMessage>,
    pub estimated_tokens: u32,
}

pub fn assemble_prompt_bundle(layers: Vec<PromptLayer>) -> Result<PromptBundle, ContextError> {
    for layer in &layers {
        validate_layer(layer)?;
    }

    let mut indexed_layers: Vec<_> = layers.into_iter().enumerate().collect();
    indexed_layers.sort_by(|(left_index, left), (right_index, right)| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left_index.cmp(right_index))
    });

    let layers: Vec<_> = indexed_layers.into_iter().map(|(_, layer)| layer).collect();
    let messages = layers
        .iter()
        .map(|layer| PromptMessage {
            role: role_for_kind(layer.kind).to_string(),
            content: layer.content.clone(),
        })
        .collect();
    let estimated_tokens = layers.iter().map(|layer| layer.token_estimate).sum();

    Ok(PromptBundle {
        layers,
        messages,
        estimated_tokens,
    })
}

fn validate_layer(layer: &PromptLayer) -> Result<(), ContextError> {
    if layer.content.trim().is_empty() {
        return Err(ContextError::InvalidLayer(
            "layer content cannot be empty".to_string(),
        ));
    }
    if layer.version == 0 {
        return Err(ContextError::InvalidLayer(
            "layer version must be greater than zero".to_string(),
        ));
    }
    validate_source(&layer.source)?;
    Ok(())
}

fn apply_overlays(
    mut layers: Vec<PromptLayer>,
    overlays: Vec<PromptOverlay>,
    profile: &PromptProfile,
    diagnostics: &mut Vec<PromptDiagnostic>,
) -> Result<Vec<PromptLayer>, ContextError> {
    let mut indexed_overlays: Vec<_> = overlays.into_iter().enumerate().collect();
    indexed_overlays.sort_by(|(left_index, left), (right_index, right)| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left_index.cmp(right_index))
    });

    for (_, overlay) in indexed_overlays {
        validate_overlay(&overlay)?;
        match overlay.action {
            PromptOverlayAction::Append => {
                let layer = overlay_layer(overlay.layer, "append overlay")?;
                validate_layer(&layer)?;
                layers.push(layer);
            }
            PromptOverlayAction::Replace { target_layer_id } => {
                let replacement = overlay_layer(overlay.layer, "replace overlay")?;
                validate_layer(&replacement)?;
                if replacement.id != target_layer_id {
                    return Err(ContextError::InvalidLayer(format!(
                        "replace overlay layer id {} must match target layer id {target_layer_id}",
                        replacement.id
                    )));
                }
                if layers
                    .iter()
                    .any(|layer| layer.id == target_layer_id && is_protected_layer(profile, layer))
                {
                    return Err(ContextError::InvalidLayer(format!(
                        "cannot replace required layer: {target_layer_id}"
                    )));
                }
                let before_len = layers.len();
                layers.retain(|layer| layer.id != target_layer_id);
                if before_len == layers.len() {
                    diagnostics.push(PromptDiagnostic::warning(
                        "replace_target_missing",
                        format!("replace overlay target not found: {target_layer_id}"),
                        Some(target_layer_id),
                    ));
                }
                layers.push(replacement);
            }
            PromptOverlayAction::Disable { target_layer_id } => {
                if layers.iter().any(|layer| {
                    layer.id == target_layer_id && is_required_hidden_layer(profile, layer)
                }) {
                    return Err(ContextError::InvalidLayer(format!(
                        "cannot disable required hidden layer: {target_layer_id}"
                    )));
                }
                let before_len = layers.len();
                layers.retain(|layer| layer.id != target_layer_id);
                if before_len == layers.len() {
                    diagnostics.push(PromptDiagnostic::warning(
                        "disable_target_missing",
                        format!("disable overlay target not found: {target_layer_id}"),
                        Some(target_layer_id),
                    ));
                }
            }
        }
    }

    Ok(layers)
}

fn validate_overlay(overlay: &PromptOverlay) -> Result<(), ContextError> {
    if overlay.id.trim().is_empty() {
        return Err(ContextError::InvalidLayer(
            "overlay id cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn overlay_layer(layer: Option<PromptLayer>, context: &str) -> Result<PromptLayer, ContextError> {
    layer.ok_or_else(|| ContextError::InvalidLayer(format!("{context} requires a layer")))
}

fn deduplicate_layers(
    layers: Vec<PromptLayer>,
    diagnostics: &mut Vec<PromptDiagnostic>,
) -> Result<Vec<PromptLayer>, ContextError> {
    let mut indexed_layers: Vec<_> = layers.into_iter().enumerate().collect();
    indexed_layers.sort_by(|(left_index, left), (right_index, right)| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left_index.cmp(right_index))
    });

    let mut seen = HashSet::new();
    let mut deduplicated = Vec::new();
    for (_, layer) in indexed_layers {
        if seen.insert(layer.id.clone()) {
            deduplicated.push(layer);
            continue;
        }

        if layer.policy == PromptLayerPolicy::Required {
            return Err(ContextError::InvalidLayer(format!(
                "duplicate required layer id: {}",
                layer.id
            )));
        }
        diagnostics.push(PromptDiagnostic::warning(
            "duplicate_optional_layer",
            format!("duplicate optional layer id skipped: {}", layer.id),
            Some(layer.id),
        ));
    }

    Ok(deduplicated)
}

fn ensure_required_layers(
    profile: &PromptProfile,
    layers: &[PromptLayer],
) -> Result<(), ContextError> {
    for required_id in &profile.required_layers {
        if !layers.iter().any(|layer| &layer.id == required_id) {
            return Err(ContextError::InvalidLayer(format!(
                "missing required layer: {required_id}"
            )));
        }
    }
    Ok(())
}

fn apply_prompt_budget(
    profile: &PromptProfile,
    layers: Vec<PromptLayer>,
    diagnostics: &mut Vec<PromptDiagnostic>,
) -> Result<(Vec<PromptLayer>, Vec<PromptLayerId>), ContextError> {
    let available_tokens =
        profile.default_budget.max_tokens - profile.default_budget.reserved_response_tokens;
    let mut used_tokens = 0;
    let mut included = Vec::new();
    let mut omitted = Vec::new();

    for layer in layers
        .iter()
        .filter(|layer| is_required_for_budget(profile, layer))
    {
        let next_used = used_tokens + layer.token_estimate;
        if next_used > available_tokens {
            return Err(ContextError::BudgetExceeded(format!(
                "required layer {} needs {} tokens with {} tokens available",
                layer.id, layer.token_estimate, available_tokens
            )));
        }
        used_tokens = next_used;
        included.push(layer.clone());
    }

    for layer in layers
        .into_iter()
        .filter(|layer| !is_required_for_budget(profile, layer))
    {
        let next_used = used_tokens + layer.token_estimate;
        if next_used <= available_tokens {
            used_tokens = next_used;
            included.push(layer);
            continue;
        }

        diagnostics.push(PromptDiagnostic::warning(
            "optional_layer_omitted",
            format!("optional layer omitted by budget: {}", layer.id),
            Some(layer.id.clone()),
        ));
        omitted.push(layer.id);
    }

    Ok((included, omitted))
}

fn is_required_for_budget(profile: &PromptProfile, layer: &PromptLayer) -> bool {
    layer.policy == PromptLayerPolicy::Required
        || profile
            .required_layers
            .iter()
            .any(|required_id| required_id == &layer.id)
}

fn is_required_hidden_layer(profile: &PromptProfile, layer: &PromptLayer) -> bool {
    layer.visibility == PromptLayerVisibility::Hidden && is_required_for_budget(profile, layer)
}

fn is_protected_layer(profile: &PromptProfile, layer: &PromptLayer) -> bool {
    is_required_for_budget(profile, layer)
}

fn validate_source(source: &PromptLayerSource) -> Result<(), ContextError> {
    match source {
        PromptLayerSource::BuiltIn => Ok(()),
        PromptLayerSource::RuntimeConfig { source } => {
            ensure_not_empty(source, "runtime config source")
        }
        PromptLayerSource::ProjectFile { path, content_hash } => {
            ensure_not_empty(path, "project file path")?;
            ensure_not_empty(content_hash, "project file content hash")
        }
        PromptLayerSource::UserPreference { key } => ensure_not_empty(key, "user preference key"),
        PromptLayerSource::Skill { skill_id, source } => {
            ensure_not_empty(skill_id, "skill id")?;
            ensure_not_empty(source, "skill source")
        }
        PromptLayerSource::Tool { tool_name } => ensure_not_empty(tool_name, "tool name"),
        PromptLayerSource::Memory { memory_id } => ensure_not_empty(memory_id, "memory id"),
        PromptLayerSource::TeamRole { role_id } => ensure_not_empty(role_id, "team role id"),
        PromptLayerSource::SessionOverlay { command_id } => {
            ensure_not_empty(command_id, "session overlay command id")
        }
    }
}

fn ensure_not_empty(value: &str, field: &str) -> Result<(), ContextError> {
    if value.trim().is_empty() {
        return Err(ContextError::InvalidLayer(format!(
            "{field} cannot be empty"
        )));
    }
    Ok(())
}

fn stable_json_hash<T>(value: &T) -> String
where
    T: Serialize,
{
    let json = serde_json::to_string(value).expect("prompt hash input serializes");
    stable_str_hash(&json)
}

fn stable_str_hash(value: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("fnv1a64:{hash:016x}")
}

fn role_for_kind(kind: PromptLayerKind) -> &'static str {
    match kind {
        PromptLayerKind::User => "user",
        PromptLayerKind::System
        | PromptLayerKind::Product
        | PromptLayerKind::Project
        | PromptLayerKind::Skill
        | PromptLayerKind::Tool
        | PromptLayerKind::Memory
        | PromptLayerKind::Runtime => "system",
    }
}
