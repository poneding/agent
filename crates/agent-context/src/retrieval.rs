use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    ContextError, ProjectContextDiagnosticSeverity, ProjectContextSummary,
    ProjectContextVisibility, ProjectFileKind, ProjectFileSummary, PromptLayer, PromptLayerKind,
    PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource, PromptLayerVisibility,
};

const PINNED_PATH_SCORE: u32 = 100;
const PATH_TOKEN_SCORE: u32 = 10;
const FILE_NAME_SCORE: u32 = 20;
const DIRECTORY_SCORE: u32 = 8;
const LANGUAGE_SCORE: u32 = 7;
const KIND_SCORE: u32 = 5;
const MAX_TOKEN_MATCHES: u32 = 5;
const MAX_PROMPT_TASK_CHARS: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextRetrievalQuery {
    pub task: String,
    pub max_candidates: u32,
    pub budget_tokens: u32,
    pub pinned_paths: Vec<String>,
}

impl ContextRetrievalQuery {
    pub fn new(
        task: impl Into<String>,
        max_candidates: u32,
        budget_tokens: u32,
        pinned_paths: Vec<String>,
    ) -> Result<Self, ContextError> {
        let task = task.into();
        let task = task.trim();
        if task.is_empty() {
            return Err(ContextError::InvalidRetrieval(
                "task cannot be empty".to_string(),
            ));
        }
        if max_candidates == 0 {
            return Err(ContextError::InvalidRetrieval(
                "max_candidates must be greater than zero".to_string(),
            ));
        }
        if budget_tokens == 0 {
            return Err(ContextError::InvalidRetrieval(
                "budget_tokens must be greater than zero".to_string(),
            ));
        }

        let mut seen = HashSet::new();
        let mut normalized_pins = Vec::with_capacity(pinned_paths.len());
        for path in pinned_paths {
            let path = normalize_path(path);
            validate_non_empty("pinned path", &path)?;
            if !seen.insert(path.clone()) {
                return Err(ContextError::InvalidRetrieval(format!(
                    "duplicate pinned path {path}"
                )));
            }
            normalized_pins.push(path);
        }

        Ok(Self {
            task: task.to_string(),
            max_candidates,
            budget_tokens,
            pinned_paths: normalized_pins,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSelectionReason {
    PinnedPath,
    PathTokenMatch,
    FileNameMatch,
    DirectoryMatch,
    LanguageMatch,
    KindMatch,
}

impl ContextSelectionReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PinnedPath => "pinned_path",
            Self::PathTokenMatch => "path_token_match",
            Self::FileNameMatch => "file_name_match",
            Self::DirectoryMatch => "directory_match",
            Self::LanguageMatch => "language_match",
            Self::KindMatch => "kind_match",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextCandidateOmissionReason {
    BudgetExceeded,
    MaxCandidates,
}

impl ContextCandidateOmissionReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BudgetExceeded => "budget_exceeded",
            Self::MaxCandidates => "max_candidates",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRetrievalDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl ContextRetrievalDiagnosticSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl From<ProjectContextDiagnosticSeverity> for ContextRetrievalDiagnosticSeverity {
    fn from(value: ProjectContextDiagnosticSeverity) -> Self {
        match value {
            ProjectContextDiagnosticSeverity::Info => Self::Info,
            ProjectContextDiagnosticSeverity::Warning => Self::Warning,
            ProjectContextDiagnosticSeverity::Error => Self::Error,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextCandidate {
    pub path: String,
    pub kind: ProjectFileKind,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub visibility: ProjectContextVisibility,
    pub score: u32,
    pub reasons: Vec<ContextSelectionReason>,
}

impl ContextCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: impl Into<String>,
        kind: ProjectFileKind,
        language: Option<String>,
        size_bytes: u64,
        line_count: Option<u32>,
        token_estimate: u32,
        visibility: ProjectContextVisibility,
        score: u32,
        reasons: Vec<ContextSelectionReason>,
    ) -> Result<Self, ContextError> {
        let path = path.into();
        validate_non_empty("candidate path", &path)?;
        if score == 0 {
            return Err(ContextError::InvalidRetrieval(
                "candidate score must be greater than zero".to_string(),
            ));
        }
        let language = normalize_optional(language, "candidate language")?;
        let mut reasons = reasons;
        reasons.sort();
        reasons.dedup();
        if reasons.is_empty() {
            return Err(ContextError::InvalidRetrieval(
                "candidate reasons cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            path,
            kind,
            language,
            size_bytes,
            line_count,
            token_estimate,
            visibility,
            score,
            reasons,
        })
    }

    fn from_file(
        file: &ProjectFileSummary,
        score: u32,
        reasons: Vec<ContextSelectionReason>,
    ) -> Result<Self, ContextError> {
        Self::new(
            file.path.clone(),
            file.kind,
            file.language.clone(),
            file.size_bytes,
            file.line_count,
            file.token_estimate,
            file.visibility,
            score,
            reasons,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextCandidateOmission {
    pub candidate: ContextCandidate,
    pub reason: ContextCandidateOmissionReason,
}

impl ContextCandidateOmission {
    pub const fn new(candidate: ContextCandidate, reason: ContextCandidateOmissionReason) -> Self {
        Self { candidate, reason }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextRetrievalDiagnostic {
    pub severity: ContextRetrievalDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

impl ContextRetrievalDiagnostic {
    pub fn new(
        severity: ContextRetrievalDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<String>,
    ) -> Result<Self, ContextError> {
        let code = code.into();
        let message = message.into();
        validate_non_empty("retrieval diagnostic code", &code)?;
        validate_non_empty("retrieval diagnostic message", &message)?;
        let path = normalize_optional(path, "retrieval diagnostic path")?;
        Ok(Self {
            severity,
            code,
            message,
            path,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextRetrievalReport {
    pub query: ContextRetrievalQuery,
    pub selected: Vec<ContextCandidate>,
    pub omitted: Vec<ContextCandidateOmission>,
    pub diagnostics: Vec<ContextRetrievalDiagnostic>,
    pub used_tokens: u32,
}

impl ContextRetrievalReport {
    pub fn new(
        query: ContextRetrievalQuery,
        selected: Vec<ContextCandidate>,
        omitted: Vec<ContextCandidateOmission>,
        mut diagnostics: Vec<ContextRetrievalDiagnostic>,
        used_tokens: u32,
    ) -> Self {
        diagnostics.sort_by(|left, right| {
            left.code
                .cmp(&right.code)
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.message.cmp(&right.message))
        });
        Self {
            query,
            selected,
            omitted,
            diagnostics,
            used_tokens,
        }
    }

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
                path: "project-context-retrieval".to_string(),
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
            "Relevant project context for task: {}",
            bounded_prompt_text(&self.query.task)
        )];
        lines.push(format!(
            "Selected {} files using {}/{} tokens.",
            self.selected.len(),
            self.used_tokens,
            self.query.budget_tokens
        ));
        if !self.selected.is_empty() {
            lines.push("Selected files:".to_string());
            for candidate in &self.selected {
                let language = candidate
                    .language
                    .as_deref()
                    .map(|language| format!(" {language}"))
                    .unwrap_or_default();
                let reasons = candidate
                    .reasons
                    .iter()
                    .map(|reason| reason.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "- {} [{}{}; {}; score={}; tokens={}; reasons={}]",
                    candidate.path,
                    candidate.kind.as_str(),
                    language,
                    candidate.visibility.as_str(),
                    candidate.score,
                    candidate.token_estimate,
                    reasons
                ));
            }
        }
        if !self.omitted.is_empty() {
            lines.push("Omitted candidates:".to_string());
            for omission in self.omitted.iter().take(40) {
                lines.push(format!(
                    "- {}: {}",
                    omission.candidate.path,
                    omission.reason.as_str()
                ));
            }
        }
        lines.join("\n")
    }
}

pub fn retrieve_relevant_context(
    summary: &ProjectContextSummary,
    query: ContextRetrievalQuery,
) -> Result<ContextRetrievalReport, ContextError> {
    let task_tokens = lexical_tokens(&query.task);
    let pinned_paths = query
        .pinned_paths
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let known_paths = summary
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<HashSet<_>>();

    let mut diagnostics = Vec::new();
    for pinned_path in &query.pinned_paths {
        if !known_paths.contains(pinned_path.as_str()) {
            diagnostics.push(diagnostic(
                ContextRetrievalDiagnosticSeverity::Warning,
                "context_retrieval.pin.unknown",
                "pinned path was not found in project summary",
                Some(pinned_path.clone()),
            ));
        }
    }

    let mut candidates = Vec::new();
    for file in &summary.files {
        let (score, reasons) = score_file(file, &task_tokens, &pinned_paths);
        if score > 0 {
            candidates.push(ContextCandidate::from_file(file, score, reasons)?);
        }
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });

    let mut selected = Vec::new();
    let mut omitted = Vec::new();
    let mut used_tokens = 0_u32;
    for candidate in candidates {
        if selected.len() >= query.max_candidates as usize {
            omitted.push(ContextCandidateOmission::new(
                candidate,
                ContextCandidateOmissionReason::MaxCandidates,
            ));
            continue;
        }
        if used_tokens.saturating_add(candidate.token_estimate) > query.budget_tokens {
            omitted.push(ContextCandidateOmission::new(
                candidate,
                ContextCandidateOmissionReason::BudgetExceeded,
            ));
            continue;
        }
        used_tokens = used_tokens.saturating_add(candidate.token_estimate);
        selected.push(candidate);
    }

    if selected.is_empty() {
        diagnostics.push(diagnostic(
            ContextRetrievalDiagnosticSeverity::Warning,
            "context_retrieval.no_candidates",
            "no relevant files were selected",
            None,
        ));
    }

    Ok(ContextRetrievalReport::new(
        query,
        selected,
        omitted,
        diagnostics,
        used_tokens,
    ))
}

fn score_file(
    file: &ProjectFileSummary,
    task_tokens: &BTreeSet<String>,
    pinned_paths: &HashSet<&str>,
) -> (u32, Vec<ContextSelectionReason>) {
    let mut score = 0_u32;
    let mut reasons = Vec::new();
    if pinned_paths.contains(file.path.as_str()) {
        score = score.saturating_add(PINNED_PATH_SCORE);
        reasons.push(ContextSelectionReason::PinnedPath);
    }

    let path_match_count = intersection_count(task_tokens, &lexical_tokens(&file.path));
    if path_match_count > 0 {
        score = score.saturating_add(PATH_TOKEN_SCORE.saturating_mul(path_match_count));
        reasons.push(ContextSelectionReason::PathTokenMatch);
    }

    let file_name_match_count = intersection_count(task_tokens, &file_name_tokens(&file.path));
    if file_name_match_count > 0 {
        score = score.saturating_add(FILE_NAME_SCORE.saturating_mul(file_name_match_count));
        reasons.push(ContextSelectionReason::FileNameMatch);
    }

    let directory_match_count = intersection_count(task_tokens, &directory_tokens(&file.path));
    if directory_match_count > 0 {
        score = score.saturating_add(DIRECTORY_SCORE.saturating_mul(directory_match_count));
        reasons.push(ContextSelectionReason::DirectoryMatch);
    }

    if let Some(language) = &file.language {
        let language_match_count = intersection_count(task_tokens, &lexical_tokens(language));
        if language_match_count > 0 {
            score = score.saturating_add(LANGUAGE_SCORE);
            reasons.push(ContextSelectionReason::LanguageMatch);
        }
    }

    let kind_match_count = intersection_count(task_tokens, &lexical_tokens(file.kind.as_str()));
    if kind_match_count > 0 {
        score = score.saturating_add(KIND_SCORE);
        reasons.push(ContextSelectionReason::KindMatch);
    }

    (score, reasons)
}

fn intersection_count(left: &BTreeSet<String>, right: &BTreeSet<String>) -> u32 {
    left.intersection(right)
        .take(MAX_TOKEN_MATCHES as usize)
        .count()
        .try_into()
        .unwrap_or(MAX_TOKEN_MATCHES)
}

fn lexical_tokens(value: &str) -> BTreeSet<String> {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }
    normalized
        .split_whitespace()
        .filter(|token| token.len() >= 2)
        .map(ToString::to_string)
        .collect()
}

fn file_name_tokens(path: &str) -> BTreeSet<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    lexical_tokens(name)
}

fn directory_tokens(path: &str) -> BTreeSet<String> {
    path.rsplit_once('/')
        .map(|(directory, _)| lexical_tokens(directory))
        .unwrap_or_default()
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
        return Err(ContextError::InvalidRetrieval(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(())
}

fn normalize_optional(value: Option<String>, label: &str) -> Result<Option<String>, ContextError> {
    value
        .map(|value| {
            let value = value.trim().to_string();
            if value.is_empty() {
                Err(ContextError::InvalidRetrieval(format!(
                    "{label} cannot be empty"
                )))
            } else {
                Ok(value)
            }
        })
        .transpose()
}

fn diagnostic(
    severity: ContextRetrievalDiagnosticSeverity,
    code: &str,
    message: &str,
    path: Option<String>,
) -> ContextRetrievalDiagnostic {
    ContextRetrievalDiagnostic::new(severity, code, message, path)
        .expect("retrieval emits valid diagnostics")
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

fn bounded_prompt_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_PROMPT_TASK_CHARS {
        redacted.chars().take(MAX_PROMPT_TASK_CHARS).collect()
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
