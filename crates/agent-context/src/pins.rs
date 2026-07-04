use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    BudgetClass, BudgetItem, ContextBudget, ContextError, ContextHybridRetrievalReport,
    ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity, ContextRetrievalReport,
    ProjectContextSummary, ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
    PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility, plan_context_budget,
};

const MAX_PROMPT_REASON_CHARS: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedProjectContextRequest {
    pub pins: Vec<PinnedProjectContextPin>,
    pub budget: ContextBudget,
    pub include_retrieval: bool,
}

impl PinnedProjectContextRequest {
    pub fn new(
        pins: Vec<PinnedProjectContextPin>,
        budget: ContextBudget,
        include_retrieval: bool,
    ) -> Result<Self, ContextError> {
        let mut seen = HashSet::new();
        let mut normalized = Vec::with_capacity(pins.len());
        for mut pin in pins {
            pin.path = normalize_path(pin.path);
            validate_non_empty("pinned context path", &pin.path)?;
            validate_non_empty("pinned context reason", &pin.reason)?;
            if !seen.insert(pin.path.clone()) {
                return Err(ContextError::InvalidPin(format!(
                    "duplicate pinned context path {}",
                    redact_secret_like_text(&pin.path)
                )));
            }
            pin.reason = bounded_text(&pin.reason, MAX_PROMPT_REASON_CHARS);
            normalized.push(pin);
        }
        plan_context_budget(budget, Vec::new())?;
        Ok(Self {
            pins: normalized,
            budget,
            include_retrieval,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedProjectContextPin {
    pub path: String,
    pub priority: PinnedContextPriority,
    pub reason: String,
}

impl PinnedProjectContextPin {
    pub fn required(
        path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, ContextError> {
        Self::new(path, PinnedContextPriority::Required, reason)
    }

    pub fn optional(
        path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, ContextError> {
        Self::new(path, PinnedContextPriority::Optional, reason)
    }

    pub fn new(
        path: impl Into<String>,
        priority: PinnedContextPriority,
        reason: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let path = normalize_path(path.into());
        let reason = reason.into().trim().to_string();
        validate_non_empty("pinned context path", &path)?;
        validate_non_empty("pinned context reason", &reason)?;
        Ok(Self {
            path,
            priority,
            reason: bounded_text(&reason, MAX_PROMPT_REASON_CHARS),
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinnedContextPriority {
    Required,
    Optional,
}

impl PinnedContextPriority {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinnedContextSource {
    ProjectFile,
    RetrievalCandidate,
    HybridRetrievalCandidate,
}

impl PinnedContextSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectFile => "project_file",
            Self::RetrievalCandidate => "retrieval_candidate",
            Self::HybridRetrievalCandidate => "hybrid_retrieval_candidate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedContextItem {
    pub path: String,
    pub kind: ProjectFileKind,
    pub language: Option<String>,
    pub visibility: ProjectContextVisibility,
    pub token_estimate: u32,
    pub priority: PinnedContextPriority,
    pub source: PinnedContextSource,
    pub reason: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinnedContextOmissionReason {
    UnknownPath,
    BudgetExceeded,
    DuplicatePath,
}

impl PinnedContextOmissionReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownPath => "unknown_path",
            Self::BudgetExceeded => "budget_exceeded",
            Self::DuplicatePath => "duplicate_path",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedContextOmission {
    pub path: String,
    pub priority: PinnedContextPriority,
    pub reason: PinnedContextOmissionReason,
    pub message: String,
}

impl PinnedContextOmission {
    fn new(
        path: impl Into<String>,
        priority: PinnedContextPriority,
        reason: PinnedContextOmissionReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: normalize_path(path.into()),
            priority,
            reason,
            message: bounded_text(&message.into(), MAX_PROMPT_REASON_CHARS),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedProjectContextRetrievalLayer {
    source: PinnedContextSource,
    layer: PromptLayer,
}

impl PinnedProjectContextRetrievalLayer {
    pub fn lexical(
        report: &ContextRetrievalReport,
        id: impl Into<String>,
        priority: i32,
    ) -> Result<Self, ContextError> {
        Ok(Self {
            source: PinnedContextSource::RetrievalCandidate,
            layer: report.to_prompt_layer(id, priority)?,
        })
    }

    pub fn hybrid(
        report: &ContextHybridRetrievalReport,
        id: impl Into<String>,
        priority: i32,
    ) -> Result<Self, ContextError> {
        Ok(Self {
            source: PinnedContextSource::HybridRetrievalCandidate,
            layer: report.to_prompt_layer(id, priority)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedProjectContextBudgetReport {
    pub request: PinnedProjectContextRequest,
    pub budget: ContextBudget,
    pub available_tokens: u32,
    pub used_tokens: u32,
    pub included: Vec<PinnedContextItem>,
    pub omitted: Vec<PinnedContextOmission>,
    pub retrieval_included: bool,
    pub retrieval_omitted_reason: Option<String>,
    pub diagnostics: Vec<ContextRetrievalDiagnostic>,
}

impl PinnedProjectContextBudgetReport {
    pub fn to_prompt_layer(
        &self,
        id: impl Into<String>,
        priority: i32,
    ) -> Result<PromptLayer, ContextError> {
        let content = self.prompt_content();
        let token_estimate = estimate_tokens(&content);
        PromptLayer::new(
            id,
            PromptLayerKind::Project,
            priority,
            content.clone(),
            token_estimate,
            PromptLayerSource::ProjectFile {
                path: "pinned-project-context".to_string(),
                content_hash: fnv1a64(&content),
            },
            PromptLayerVisibility::SummaryOnly,
            PromptLayerSensitivity::Workspace,
            PromptLayerPolicy::Optional,
            1,
        )
    }

    fn prompt_content(&self) -> String {
        let mut lines = vec![format!(
            "Pinned project context budget: used {}/{} tokens.",
            self.used_tokens, self.available_tokens
        )];
        lines.push(format!(
            "Retrieval context: {}",
            if self.retrieval_included {
                "included".to_string()
            } else {
                self.retrieval_omitted_reason
                    .as_ref()
                    .map(|reason| format!("omitted:{reason}"))
                    .unwrap_or_else(|| "not_requested".to_string())
            }
        ));
        if !self.included.is_empty() {
            lines.push("Included pinned files:".to_string());
            for item in &self.included {
                let language = item
                    .language
                    .as_deref()
                    .map(|language| format!(" {language}"))
                    .unwrap_or_default();
                lines.push(format!(
                    "- {} [{}{}; {}; priority={}; source={}; tokens={}; reason={}]",
                    item.path,
                    item.kind.as_str(),
                    language,
                    item.visibility.as_str(),
                    item.priority.as_str(),
                    item.source.as_str(),
                    item.token_estimate,
                    bounded_text(&item.reason, MAX_PROMPT_REASON_CHARS)
                ));
            }
        }
        if !self.omitted.is_empty() {
            lines.push("Omitted pinned files:".to_string());
            for omission in &self.omitted {
                lines.push(format!(
                    "- {} [{}; priority={}; message={}]",
                    omission.path,
                    omission.reason.as_str(),
                    omission.priority.as_str(),
                    bounded_text(&omission.message, MAX_PROMPT_REASON_CHARS)
                ));
            }
        }
        lines.join("\n")
    }
}

#[derive(Clone, Debug)]
struct PinBudgetEntry {
    item: PinnedContextItem,
    layer: PromptLayer,
}

pub fn plan_pinned_project_context_budget(
    summary: &ProjectContextSummary,
    request: PinnedProjectContextRequest,
    retrieval: Option<PinnedProjectContextRetrievalLayer>,
) -> Result<PinnedProjectContextBudgetReport, ContextError> {
    let files = summary
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let mut required = Vec::new();
    let mut optional = Vec::new();
    let mut omitted = Vec::new();
    let mut diagnostics = Vec::new();

    for (index, pin) in request.pins.iter().enumerate() {
        let Some(file) = files.get(pin.path.as_str()) else {
            match pin.priority {
                PinnedContextPriority::Required => {
                    return Err(ContextError::InvalidPin(format!(
                        "required pinned context path {} was not found",
                        redact_secret_like_text(&pin.path)
                    )));
                }
                PinnedContextPriority::Optional => {
                    omitted.push(PinnedContextOmission::new(
                        pin.path.clone(),
                        pin.priority,
                        PinnedContextOmissionReason::UnknownPath,
                        "optional pinned context path was not found",
                    ));
                    diagnostics.push(diagnostic(
                        ContextRetrievalDiagnosticSeverity::Warning,
                        "context_pins.optional.unknown",
                        "optional pinned context path was not found",
                        Some(pin.path.clone()),
                    ));
                    continue;
                }
            }
        };
        let entry = pin_budget_entry(index, pin, file)?;
        match pin.priority {
            PinnedContextPriority::Required => required.push(entry),
            PinnedContextPriority::Optional => optional.push(entry),
        }
    }

    let mut entries = BTreeMap::new();
    let mut budget_items = Vec::new();
    for entry in required {
        entries.insert(entry.layer.id.clone(), entry.item.clone());
        budget_items.push(BudgetItem {
            layer: entry.layer,
            class: BudgetClass::Required,
        });
    }
    for entry in optional {
        entries.insert(entry.layer.id.clone(), entry.item.clone());
        budget_items.push(BudgetItem {
            layer: entry.layer,
            class: BudgetClass::Optional,
        });
    }

    let retrieval_requested = request.include_retrieval;
    let retrieval_layer_id = retrieval
        .as_ref()
        .map(|retrieval| retrieval.layer.id.clone());
    if request.include_retrieval {
        if let Some(retrieval) = retrieval {
            let _source = retrieval.source;
            budget_items.push(BudgetItem {
                layer: retrieval.layer,
                class: BudgetClass::Optional,
            });
        } else {
            diagnostics.push(diagnostic(
                ContextRetrievalDiagnosticSeverity::Info,
                "context_pins.retrieval.unavailable",
                "retrieval context was requested but no retrieval layer was provided",
                None,
            ));
        }
    }

    let plan = plan_context_budget(request.budget, budget_items)?;
    let mut included = Vec::new();
    let mut retrieval_included = false;
    for layer in &plan.included {
        if retrieval_layer_id
            .as_ref()
            .is_some_and(|id| id == &layer.id)
        {
            retrieval_included = true;
            continue;
        }
        if let Some(item) = entries.remove(&layer.id) {
            included.push(item);
        }
    }

    let mut retrieval_omitted_reason = if retrieval_requested && retrieval_layer_id.is_none() {
        Some("not_provided".to_string())
    } else {
        None
    };
    for (layer, reason) in &plan.omitted {
        if retrieval_layer_id
            .as_ref()
            .is_some_and(|id| id == &layer.id)
        {
            retrieval_omitted_reason = Some(reason.clone());
            diagnostics.push(diagnostic(
                ContextRetrievalDiagnosticSeverity::Info,
                "context_pins.retrieval.budget_exceeded",
                "retrieval context was omitted because the pinned context budget was exhausted",
                None,
            ));
            continue;
        }
        if let Some(item) = entries.remove(&layer.id) {
            omitted.push(PinnedContextOmission::new(
                item.path,
                item.priority,
                PinnedContextOmissionReason::BudgetExceeded,
                reason,
            ));
            diagnostics.push(diagnostic(
                ContextRetrievalDiagnosticSeverity::Info,
                "context_pins.optional.budget_exceeded",
                "optional pinned context was omitted because the pinned context budget was exhausted",
                Some(layer.source_path().unwrap_or_else(|| layer.id.to_string())),
            ));
        }
    }

    diagnostics.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.message.cmp(&right.message))
    });

    Ok(PinnedProjectContextBudgetReport {
        request: request.clone(),
        budget: request.budget,
        available_tokens: plan.available_tokens,
        used_tokens: plan.used_tokens,
        included,
        omitted,
        retrieval_included,
        retrieval_omitted_reason,
        diagnostics,
    })
}

fn pin_budget_entry(
    index: usize,
    pin: &PinnedProjectContextPin,
    file: &ProjectFileSummary,
) -> Result<PinBudgetEntry, ContextError> {
    let item = PinnedContextItem {
        path: file.path.clone(),
        kind: file.kind,
        language: file.language.clone(),
        visibility: file.visibility,
        token_estimate: file.token_estimate,
        priority: pin.priority,
        source: PinnedContextSource::ProjectFile,
        reason: pin.reason.clone(),
    };
    let layer = pin_prompt_layer(index, &item)?;
    Ok(PinBudgetEntry { item, layer })
}

fn pin_prompt_layer(index: usize, item: &PinnedContextItem) -> Result<PromptLayer, ContextError> {
    let language = item
        .language
        .as_deref()
        .map(|language| format!("language: {language}\n"))
        .unwrap_or_default();
    let content = format!(
        "Pinned project context\npath: {}\nkind: {}\n{}visibility: {}\ntoken_estimate: {}\npriority: {}\nreason: {}",
        item.path,
        item.kind.as_str(),
        language,
        item.visibility.as_str(),
        item.token_estimate,
        item.priority.as_str(),
        bounded_text(&item.reason, MAX_PROMPT_REASON_CHARS)
    );
    PromptLayer::new(
        format!(
            "pinned-context:{}:{}:{}",
            item.priority.as_str(),
            index,
            item.path
        ),
        PromptLayerKind::Project,
        80 - i32::try_from(index).unwrap_or(80),
        content,
        item.token_estimate,
        PromptLayerSource::ProjectFile {
            path: item.path.clone(),
            content_hash: fnv1a64(&item.path),
        },
        PromptLayerVisibility::SummaryOnly,
        PromptLayerSensitivity::Workspace,
        PromptLayerPolicy::Required,
        1,
    )
}

trait PromptLayerPath {
    fn source_path(&self) -> Option<String>;
}

impl PromptLayerPath for PromptLayer {
    fn source_path(&self) -> Option<String> {
        match &self.source {
            PromptLayerSource::ProjectFile { path, .. } => Some(path.clone()),
            _ => None,
        }
    }
}

fn diagnostic(
    severity: ContextRetrievalDiagnosticSeverity,
    code: &str,
    message: &str,
    path: Option<String>,
) -> ContextRetrievalDiagnostic {
    ContextRetrievalDiagnostic::new(severity, code, message, path)
        .expect("pinned context emits valid diagnostics")
}

fn normalize_path(path: impl Into<String>) -> String {
    let mut path = path.into().trim().replace('\\', "/");
    while let Some(stripped) = path.strip_prefix("./") {
        path = stripped.to_string();
    }
    path
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), ContextError> {
    if value.trim().is_empty() {
        return Err(ContextError::InvalidPin(format!("{label} cannot be empty")));
    }
    Ok(())
}

fn estimate_tokens(content: &str) -> u32 {
    let chars = content.chars().count();
    chars.div_ceil(4).max(1).try_into().unwrap_or(u32::MAX)
}

fn fnv1a64(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn bounded_text(value: &str, max_chars: usize) -> String {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > max_chars {
        redacted.chars().take(max_chars).collect()
    } else {
        redacted
    }
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("secret")
                || lower.contains("token")
                || lower.contains("password")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
