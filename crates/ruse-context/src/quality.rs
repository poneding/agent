use serde::{Deserialize, Serialize};

use crate::{
    ContextError, ContextHybridRetrievalReport, ContextRetrievalDiagnostic,
    ContextRetrievalDiagnosticSeverity, ContextRetrievalReport, ContextSemanticRetrievalStatus,
    PinnedContextOmissionReason, PinnedContextPriority, PinnedProjectContextBudgetReport,
};

const MAX_QUALITY_TEXT_CHARS: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextQualityTrace {
    pub task: String,
    pub workspace_root_display: String,
    pub retrieval_kind: ContextQualityRetrievalKind,
    pub selected_count: u32,
    pub omitted_count: u32,
    pub pinned_included_count: u32,
    pub pinned_omitted_count: u32,
    pub used_tokens: u32,
    pub available_tokens: u32,
    pub budget_pressure: ContextQualityBudgetPressure,
    pub signals: Vec<ContextQualitySignal>,
    pub diagnostics: Vec<ContextQualityDiagnostic>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextQualityRetrievalKind {
    Lexical,
    Hybrid,
    PinnedLexical,
    PinnedHybrid,
}

impl ContextQualityRetrievalKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::Hybrid => "hybrid",
            Self::PinnedLexical => "pinned_lexical",
            Self::PinnedHybrid => "pinned_hybrid",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextQualityBudgetPressure {
    Low,
    Medium,
    High,
    Exhausted,
}

impl ContextQualityBudgetPressure {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Exhausted => "exhausted",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextQualitySignal {
    pub path: Option<String>,
    pub kind: ContextQualitySignalKind,
    pub weight: u32,
    pub message: String,
}

impl ContextQualitySignal {
    pub fn new(
        path: Option<String>,
        kind: ContextQualitySignalKind,
        weight: u32,
        message: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let path = normalize_optional(path)?;
        let message = bounded_redacted_text(&message.into());
        validate_non_empty("context quality signal message", &message)?;
        Ok(Self {
            path,
            kind,
            weight,
            message,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextQualitySignalKind {
    SelectedCandidate,
    OmittedCandidate,
    IncludedRequiredPin,
    IncludedOptionalPin,
    OmittedOptionalPin,
    RetrievalLayerIncluded,
    RetrievalLayerOmitted,
    SemanticCompleted,
    SemanticFallback,
}

impl ContextQualitySignalKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelectedCandidate => "selected_candidate",
            Self::OmittedCandidate => "omitted_candidate",
            Self::IncludedRequiredPin => "included_required_pin",
            Self::IncludedOptionalPin => "included_optional_pin",
            Self::OmittedOptionalPin => "omitted_optional_pin",
            Self::RetrievalLayerIncluded => "retrieval_layer_included",
            Self::RetrievalLayerOmitted => "retrieval_layer_omitted",
            Self::SemanticCompleted => "semantic_completed",
            Self::SemanticFallback => "semantic_fallback",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextQualityDiagnostic {
    pub severity: ContextQualityDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

impl ContextQualityDiagnostic {
    pub fn new(
        severity: ContextQualityDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<String>,
    ) -> Result<Self, ContextError> {
        let code = bounded_redacted_text(&code.into());
        let message = bounded_redacted_text(&message.into());
        validate_non_empty("context quality diagnostic code", &code)?;
        validate_non_empty("context quality diagnostic message", &message)?;
        let path = normalize_optional(path)?;
        Ok(Self {
            severity,
            code,
            message,
            path,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextQualityDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl ContextQualityDiagnosticSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl From<ContextRetrievalDiagnosticSeverity> for ContextQualityDiagnosticSeverity {
    fn from(value: ContextRetrievalDiagnosticSeverity) -> Self {
        match value {
            ContextRetrievalDiagnosticSeverity::Info => Self::Info,
            ContextRetrievalDiagnosticSeverity::Warning => Self::Warning,
            ContextRetrievalDiagnosticSeverity::Error => Self::Error,
        }
    }
}

pub fn build_context_quality_trace_from_retrieval(
    workspace_root_display: impl Into<String>,
    report: &ContextRetrievalReport,
) -> Result<ContextQualityTrace, ContextError> {
    let mut signals = Vec::new();
    for candidate in &report.selected {
        signals.push(ContextQualitySignal::new(
            Some(candidate.path.clone()),
            ContextQualitySignalKind::SelectedCandidate,
            candidate.score,
            format!(
                "selected candidate score={} tokens={}",
                candidate.score, candidate.token_estimate
            ),
        )?);
    }
    for omission in &report.omitted {
        signals.push(ContextQualitySignal::new(
            Some(omission.candidate.path.clone()),
            ContextQualitySignalKind::OmittedCandidate,
            omission.candidate.score,
            format!("omitted candidate reason={}", omission.reason.as_str()),
        )?);
    }
    build_trace(
        workspace_root_display,
        &report.query.task,
        ContextQualityRetrievalKind::Lexical,
        report.selected.len(),
        report.omitted.len(),
        0,
        0,
        report.used_tokens,
        report.query.budget_tokens,
        signals,
        diagnostics_from_retrieval(&report.diagnostics)?,
    )
}

pub fn build_context_quality_trace_from_hybrid(
    workspace_root_display: impl Into<String>,
    report: &ContextHybridRetrievalReport,
) -> Result<ContextQualityTrace, ContextError> {
    let mut signals = hybrid_signals(report)?;
    for candidate in &report.selected {
        signals.push(ContextQualitySignal::new(
            Some(candidate.candidate.path.clone()),
            ContextQualitySignalKind::SelectedCandidate,
            candidate.candidate.score,
            format!(
                "selected candidate score={} tokens={}",
                candidate.candidate.score, candidate.candidate.token_estimate
            ),
        )?);
    }
    for omission in &report.omitted {
        signals.push(ContextQualitySignal::new(
            Some(omission.candidate.candidate.path.clone()),
            ContextQualitySignalKind::OmittedCandidate,
            omission.candidate.candidate.score,
            format!("omitted candidate reason={}", omission.reason.as_str()),
        )?);
    }
    build_trace(
        workspace_root_display,
        &report.query.task,
        ContextQualityRetrievalKind::Hybrid,
        report.selected.len(),
        report.omitted.len(),
        0,
        0,
        report.used_tokens,
        report.query.budget_tokens,
        signals,
        diagnostics_from_retrieval(&report.diagnostics)?,
    )
}

pub fn build_context_quality_trace_from_pinned_retrieval(
    workspace_root_display: impl Into<String>,
    retrieval: &ContextRetrievalReport,
    pinned: &PinnedProjectContextBudgetReport,
) -> Result<ContextQualityTrace, ContextError> {
    let mut signals = pinned_signals(pinned)?;
    for candidate in &retrieval.selected {
        signals.push(ContextQualitySignal::new(
            Some(candidate.path.clone()),
            ContextQualitySignalKind::SelectedCandidate,
            candidate.score,
            format!(
                "selected candidate score={} tokens={}",
                candidate.score, candidate.token_estimate
            ),
        )?);
    }
    build_trace(
        workspace_root_display,
        &retrieval.query.task,
        ContextQualityRetrievalKind::PinnedLexical,
        retrieval.selected.len(),
        retrieval.omitted.len(),
        pinned.included.len(),
        pinned.omitted.len(),
        pinned.used_tokens,
        pinned.available_tokens,
        signals,
        diagnostics_from_retrieval(&pinned.diagnostics)?,
    )
}

pub fn build_context_quality_trace_from_pinned_hybrid(
    workspace_root_display: impl Into<String>,
    hybrid: &ContextHybridRetrievalReport,
    pinned: &PinnedProjectContextBudgetReport,
) -> Result<ContextQualityTrace, ContextError> {
    let mut signals = hybrid_signals(hybrid)?;
    signals.extend(pinned_signals(pinned)?);
    for candidate in &hybrid.selected {
        signals.push(ContextQualitySignal::new(
            Some(candidate.candidate.path.clone()),
            ContextQualitySignalKind::SelectedCandidate,
            candidate.candidate.score,
            format!(
                "selected candidate score={} tokens={}",
                candidate.candidate.score, candidate.candidate.token_estimate
            ),
        )?);
    }
    build_trace(
        workspace_root_display,
        &hybrid.query.task,
        ContextQualityRetrievalKind::PinnedHybrid,
        hybrid.selected.len(),
        hybrid.omitted.len(),
        pinned.included.len(),
        pinned.omitted.len(),
        pinned.used_tokens,
        pinned.available_tokens,
        signals,
        diagnostics_from_retrieval(&pinned.diagnostics)?,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_trace(
    workspace_root_display: impl Into<String>,
    task: &str,
    retrieval_kind: ContextQualityRetrievalKind,
    selected_count: usize,
    omitted_count: usize,
    pinned_included_count: usize,
    pinned_omitted_count: usize,
    used_tokens: u32,
    available_tokens: u32,
    mut signals: Vec<ContextQualitySignal>,
    mut diagnostics: Vec<ContextQualityDiagnostic>,
) -> Result<ContextQualityTrace, ContextError> {
    let workspace_root_display = bounded_redacted_text(&workspace_root_display.into());
    validate_non_empty("context quality workspace root", &workspace_root_display)?;
    let task = bounded_redacted_text(task);
    validate_non_empty("context quality task", &task)?;
    signals.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| right.weight.cmp(&left.weight))
            .then_with(|| left.message.cmp(&right.message))
    });
    diagnostics.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.message.cmp(&right.message))
    });
    Ok(ContextQualityTrace {
        task,
        workspace_root_display,
        retrieval_kind,
        selected_count: selected_count.try_into().map_err(|_| {
            ContextError::InvalidRetrieval("too many selected context items".to_string())
        })?,
        omitted_count: omitted_count.try_into().map_err(|_| {
            ContextError::InvalidRetrieval("too many omitted context items".to_string())
        })?,
        pinned_included_count: pinned_included_count.try_into().map_err(|_| {
            ContextError::InvalidRetrieval("too many included pinned context items".to_string())
        })?,
        pinned_omitted_count: pinned_omitted_count.try_into().map_err(|_| {
            ContextError::InvalidRetrieval("too many omitted pinned context items".to_string())
        })?,
        used_tokens,
        available_tokens,
        budget_pressure: budget_pressure(used_tokens, available_tokens),
        signals,
        diagnostics,
    })
}

fn hybrid_signals(
    report: &ContextHybridRetrievalReport,
) -> Result<Vec<ContextQualitySignal>, ContextError> {
    let semantic_kind = match report.semantic.status {
        ContextSemanticRetrievalStatus::Completed => ContextQualitySignalKind::SemanticCompleted,
        ContextSemanticRetrievalStatus::Disabled
        | ContextSemanticRetrievalStatus::FallbackLexical => {
            ContextQualitySignalKind::SemanticFallback
        }
    };
    Ok(vec![ContextQualitySignal::new(
        None,
        semantic_kind,
        report.semantic.embedded_document_count,
        format!(
            "semantic status={} embedded={}/{}",
            report.semantic.status.as_str(),
            report.semantic.embedded_document_count,
            report.semantic.document_count
        ),
    )?])
}

fn pinned_signals(
    pinned: &PinnedProjectContextBudgetReport,
) -> Result<Vec<ContextQualitySignal>, ContextError> {
    let mut signals = Vec::new();
    for item in &pinned.included {
        let kind = match item.priority {
            PinnedContextPriority::Required => ContextQualitySignalKind::IncludedRequiredPin,
            PinnedContextPriority::Optional => ContextQualitySignalKind::IncludedOptionalPin,
        };
        signals.push(ContextQualitySignal::new(
            Some(item.path.clone()),
            kind,
            item.token_estimate,
            format!(
                "included pinned context priority={} tokens={} reason={}",
                item.priority.as_str(),
                item.token_estimate,
                item.reason
            ),
        )?);
    }
    for omission in &pinned.omitted {
        signals.push(ContextQualitySignal::new(
            Some(omission.path.clone()),
            ContextQualitySignalKind::OmittedOptionalPin,
            omission_weight(omission.reason),
            format!(
                "omitted pinned context reason={} priority={} message={}",
                omission.reason.as_str(),
                omission.priority.as_str(),
                omission.message
            ),
        )?);
    }
    let (kind, message) = if pinned.retrieval_included {
        (
            ContextQualitySignalKind::RetrievalLayerIncluded,
            "retrieval context included".to_string(),
        )
    } else {
        (
            ContextQualitySignalKind::RetrievalLayerOmitted,
            format!(
                "retrieval context omitted reason={}",
                pinned
                    .retrieval_omitted_reason
                    .as_deref()
                    .unwrap_or("not_requested")
            ),
        )
    };
    signals.push(ContextQualitySignal::new(None, kind, 0, message)?);
    Ok(signals)
}

fn diagnostics_from_retrieval(
    diagnostics: &[ContextRetrievalDiagnostic],
) -> Result<Vec<ContextQualityDiagnostic>, ContextError> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            ContextQualityDiagnostic::new(
                diagnostic.severity.into(),
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                diagnostic.path.clone(),
            )
        })
        .collect()
}

const fn omission_weight(reason: PinnedContextOmissionReason) -> u32 {
    match reason {
        PinnedContextOmissionReason::UnknownPath => 10,
        PinnedContextOmissionReason::BudgetExceeded => 20,
        PinnedContextOmissionReason::DuplicatePath => 30,
    }
}

const fn budget_pressure(used_tokens: u32, available_tokens: u32) -> ContextQualityBudgetPressure {
    if available_tokens == 0 || used_tokens >= available_tokens {
        return ContextQualityBudgetPressure::Exhausted;
    }
    let ratio = used_tokens.saturating_mul(100) / available_tokens;
    if ratio >= 80 {
        ContextQualityBudgetPressure::High
    } else if ratio >= 40 {
        ContextQualityBudgetPressure::Medium
    } else {
        ContextQualityBudgetPressure::Low
    }
}

fn normalize_optional(value: Option<String>) -> Result<Option<String>, ContextError> {
    value
        .map(|value| {
            let value = bounded_redacted_text(&value);
            validate_non_empty("context quality optional text", &value)?;
            Ok(value)
        })
        .transpose()
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), ContextError> {
    if value.trim().is_empty() {
        return Err(ContextError::InvalidRetrieval(format!(
            "{field} cannot be empty"
        )));
    }
    Ok(())
}

fn bounded_redacted_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value.trim());
    if redacted.chars().count() > MAX_QUALITY_TEXT_CHARS {
        redacted.chars().take(MAX_QUALITY_TEXT_CHARS).collect()
    } else {
        redacted
    }
}

fn redact_secret_like_text(value: &str) -> String {
    let mut output = Vec::new();
    for token in value.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if lower.contains("api_key")
            || lower.contains("secret")
            || lower == "token"
            || lower.contains("access_token")
            || lower.contains("auth_token")
            || lower.starts_with("sk-")
        {
            output.push("[redacted]");
        } else {
            output.push(token);
        }
    }
    output.join(" ")
}
