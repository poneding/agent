use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use ruse_context::{
    ContextBudget, ContextCandidate, ContextCandidateOmission, ContextCandidateOmissionReason,
    ContextHybridCandidate, ContextHybridCandidateOmission, ContextHybridRetrievalReport,
    ContextQualityBudgetPressure, ContextQualityDiagnostic, ContextQualityDiagnosticSeverity,
    ContextQualityRetrievalKind, ContextQualitySignal, ContextQualitySignalKind,
    ContextQualityTrace, ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity,
    ContextRetrievalQuery, ContextRetrievalReport, ContextSelectionReason,
    ContextSemanticFallbackPolicy, ContextSemanticRetrievalConfig, ContextSemanticRetrievalMode,
    ContextSemanticRetrievalStatus, ContextSemanticRetrievalSummary, PinnedContextItem,
    PinnedContextOmission, PinnedContextOmissionReason, PinnedContextPriority, PinnedContextSource,
    PinnedProjectContextBudgetReport, PinnedProjectContextPin, PinnedProjectContextRequest,
    ProjectContextDiagnostic, ProjectContextDiagnosticSeverity, ProjectContextLimits,
    ProjectContextOmission, ProjectContextSummary, ProjectContextVisibility,
    ProjectDirectorySummary, ProjectFileKind, ProjectFileSummary, ProjectSemanticDocument,
};
use ruse_context::{
    PinnedProjectContextRetrievalLayer, build_context_quality_trace_from_hybrid,
    build_context_quality_trace_from_pinned_hybrid,
    build_context_quality_trace_from_pinned_retrieval, build_context_quality_trace_from_retrieval,
    plan_pinned_project_context_budget, retrieve_hybrid_context, retrieve_relevant_context,
};
use ruse_memory::EmbeddingProvider;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContextScanInput {
    pub workspace_root: PathBuf,
    pub limits: ProjectContextLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContextScanReport {
    pub summary: ProjectContextSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContextRetrievalInput {
    pub workspace_root: PathBuf,
    pub limits: ProjectContextLimits,
    pub query: ContextRetrievalQuery,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextRetrievalReport {
    pub summary: ProjectContextSummary,
    pub retrieval: ContextRetrievalReport,
}

pub struct ProjectContextHybridRetrievalInput {
    pub workspace_root: PathBuf,
    pub limits: ProjectContextLimits,
    pub query: ContextRetrievalQuery,
    pub semantic_config: ContextSemanticRetrievalConfig,
    pub embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextHybridRetrievalReport {
    pub summary: ProjectContextSummary,
    pub hybrid: ContextHybridRetrievalReport,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextQualityRetrievalReport {
    pub summary: ProjectContextSummary,
    pub retrieval: ContextRetrievalReport,
    pub quality: ContextQualityTrace,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextHybridQualityRetrievalReport {
    pub summary: ProjectContextSummary,
    pub hybrid: ContextHybridRetrievalReport,
    pub quality: ContextQualityTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContextPinnedRetrievalInput {
    pub workspace_root: PathBuf,
    pub limits: ProjectContextLimits,
    pub query: ContextRetrievalQuery,
    pub pinned_request: PinnedProjectContextRequest,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextPinnedRetrievalReport {
    pub summary: ProjectContextSummary,
    pub retrieval: ContextRetrievalReport,
    pub pinned: PinnedProjectContextBudgetReport,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextPinnedQualityRetrievalReport {
    pub summary: ProjectContextSummary,
    pub retrieval: ContextRetrievalReport,
    pub pinned: PinnedProjectContextBudgetReport,
    pub quality: ContextQualityTrace,
}

pub struct ProjectContextHybridPinnedRetrievalInput {
    pub workspace_root: PathBuf,
    pub limits: ProjectContextLimits,
    pub query: ContextRetrievalQuery,
    pub semantic_config: ContextSemanticRetrievalConfig,
    pub embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    pub pinned_request: PinnedProjectContextRequest,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextHybridPinnedRetrievalReport {
    pub summary: ProjectContextSummary,
    pub hybrid: ContextHybridRetrievalReport,
    pub pinned: PinnedProjectContextBudgetReport,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectContextHybridPinnedQualityRetrievalReport {
    pub summary: ProjectContextSummary,
    pub hybrid: ContextHybridRetrievalReport,
    pub pinned: PinnedProjectContextBudgetReport,
    pub quality: ContextQualityTrace,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ProjectContextScanError {
    #[error("workspace root is invalid")]
    InvalidWorkspace {
        diagnostics: Vec<ProjectContextDiagnostic>,
    },
    #[error("project context scan failed")]
    Scan {
        diagnostics: Vec<ProjectContextDiagnostic>,
    },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ProjectContextRetrievalError {
    #[error("workspace root is invalid")]
    InvalidWorkspace {
        diagnostics: Vec<ProjectContextDiagnostic>,
    },
    #[error("project context scan failed")]
    Scan {
        diagnostics: Vec<ProjectContextDiagnostic>,
    },
    #[error("project context retrieval failed")]
    Retrieval {
        diagnostics: Vec<ProjectContextDiagnostic>,
    },
}

impl ProjectContextScanError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidWorkspace { .. } => "project_context.workspace.invalid",
            Self::Scan { .. } => "project_context.scan.failed",
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[ProjectContextDiagnostic] {
        match self {
            Self::InvalidWorkspace { diagnostics } | Self::Scan { diagnostics } => diagnostics,
        }
    }
}

impl ProjectContextRetrievalError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidWorkspace { .. } => "project_context.workspace.invalid",
            Self::Scan { .. } => "project_context.scan.failed",
            Self::Retrieval { .. } => "project_context.retrieval.failed",
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[ProjectContextDiagnostic] {
        match self {
            Self::InvalidWorkspace { diagnostics }
            | Self::Scan { diagnostics }
            | Self::Retrieval { diagnostics } => diagnostics,
        }
    }
}

#[derive(Default)]
struct ScanState {
    files: Vec<ProjectFileSummary>,
    directory_paths: HashSet<String>,
    omitted: Vec<ProjectContextOmission>,
    diagnostics: Vec<ProjectContextDiagnostic>,
    used_bytes: u64,
    max_files_reported: bool,
}

pub fn scan_project_context(
    input: ProjectContextScanInput,
) -> Result<ProjectContextScanReport, ProjectContextScanError> {
    let workspace_root = validate_workspace_root(&input.workspace_root)?;
    let mut state = ScanState::default();
    scan_dir(
        &workspace_root,
        &workspace_root,
        0,
        input.limits,
        &mut state,
    );
    let directories = directory_summaries(&state.directory_paths, &state.files)
        .map_err(scan_error_from_context)?;
    let summary = ProjectContextSummary::new(
        ".",
        state.files,
        directories,
        state.omitted,
        state.diagnostics,
        input.limits,
    )
    .map_err(scan_error_from_context)?;

    Ok(ProjectContextScanReport { summary })
}

pub fn retrieve_project_context(
    input: ProjectContextRetrievalInput,
) -> Result<ProjectContextRetrievalReport, ProjectContextRetrievalError> {
    let scan_report = scan_project_context(ProjectContextScanInput {
        workspace_root: input.workspace_root,
        limits: input.limits,
    })
    .map_err(retrieval_error_from_scan)?;
    let retrieval = retrieve_relevant_context(&scan_report.summary, input.query)
        .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextRetrievalReport {
        summary: scan_report.summary,
        retrieval,
    })
}

pub async fn retrieve_project_context_hybrid(
    input: ProjectContextHybridRetrievalInput,
) -> Result<ProjectContextHybridRetrievalReport, ProjectContextRetrievalError> {
    let scan_report = scan_project_context(ProjectContextScanInput {
        workspace_root: input.workspace_root,
        limits: input.limits,
    })
    .map_err(retrieval_error_from_scan)?;
    let hybrid = retrieve_hybrid_context(
        &scan_report.summary,
        input.query,
        input.semantic_config,
        input.embedding_provider.as_deref(),
    )
    .await
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextHybridRetrievalReport {
        summary: scan_report.summary,
        hybrid,
    })
}

pub fn retrieve_project_context_quality(
    input: ProjectContextRetrievalInput,
) -> Result<ProjectContextQualityRetrievalReport, ProjectContextRetrievalError> {
    let report = retrieve_project_context(input)?;
    let quality = build_context_quality_trace_from_retrieval(
        &report.summary.workspace_root_display,
        &report.retrieval,
    )
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextQualityRetrievalReport {
        summary: report.summary,
        retrieval: report.retrieval,
        quality,
    })
}

pub async fn retrieve_project_context_hybrid_quality(
    input: ProjectContextHybridRetrievalInput,
) -> Result<ProjectContextHybridQualityRetrievalReport, ProjectContextRetrievalError> {
    let report = retrieve_project_context_hybrid(input).await?;
    let quality = build_context_quality_trace_from_hybrid(
        &report.summary.workspace_root_display,
        &report.hybrid,
    )
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextHybridQualityRetrievalReport {
        summary: report.summary,
        hybrid: report.hybrid,
        quality,
    })
}

pub fn retrieve_project_context_pinned(
    input: ProjectContextPinnedRetrievalInput,
) -> Result<ProjectContextPinnedRetrievalReport, ProjectContextRetrievalError> {
    let scan_report = scan_project_context(ProjectContextScanInput {
        workspace_root: input.workspace_root,
        limits: input.limits,
    })
    .map_err(retrieval_error_from_scan)?;
    let retrieval = retrieve_relevant_context(&scan_report.summary, input.query)
        .map_err(retrieval_error_from_context)?;
    let retrieval_layer = if input.pinned_request.include_retrieval {
        Some(
            PinnedProjectContextRetrievalLayer::lexical(
                &retrieval,
                "project-context-retrieval",
                60,
            )
            .map_err(retrieval_error_from_context)?,
        )
    } else {
        None
    };
    let pinned = plan_pinned_project_context_budget(
        &scan_report.summary,
        input.pinned_request,
        retrieval_layer,
    )
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextPinnedRetrievalReport {
        summary: scan_report.summary,
        retrieval,
        pinned,
    })
}

pub fn retrieve_project_context_pinned_quality(
    input: ProjectContextPinnedRetrievalInput,
) -> Result<ProjectContextPinnedQualityRetrievalReport, ProjectContextRetrievalError> {
    let report = retrieve_project_context_pinned(input)?;
    let quality = build_context_quality_trace_from_pinned_retrieval(
        &report.summary.workspace_root_display,
        &report.retrieval,
        &report.pinned,
    )
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextPinnedQualityRetrievalReport {
        summary: report.summary,
        retrieval: report.retrieval,
        pinned: report.pinned,
        quality,
    })
}

pub async fn retrieve_project_context_hybrid_pinned(
    input: ProjectContextHybridPinnedRetrievalInput,
) -> Result<ProjectContextHybridPinnedRetrievalReport, ProjectContextRetrievalError> {
    let scan_report = scan_project_context(ProjectContextScanInput {
        workspace_root: input.workspace_root,
        limits: input.limits,
    })
    .map_err(retrieval_error_from_scan)?;
    let hybrid = retrieve_hybrid_context(
        &scan_report.summary,
        input.query,
        input.semantic_config,
        input.embedding_provider.as_deref(),
    )
    .await
    .map_err(retrieval_error_from_context)?;
    let retrieval_layer = if input.pinned_request.include_retrieval {
        Some(
            PinnedProjectContextRetrievalLayer::hybrid(
                &hybrid,
                "project-context-hybrid-retrieval",
                60,
            )
            .map_err(retrieval_error_from_context)?,
        )
    } else {
        None
    };
    let pinned = plan_pinned_project_context_budget(
        &scan_report.summary,
        input.pinned_request,
        retrieval_layer,
    )
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextHybridPinnedRetrievalReport {
        summary: scan_report.summary,
        hybrid,
        pinned,
    })
}

pub async fn retrieve_project_context_hybrid_pinned_quality(
    input: ProjectContextHybridPinnedRetrievalInput,
) -> Result<ProjectContextHybridPinnedQualityRetrievalReport, ProjectContextRetrievalError> {
    let report = retrieve_project_context_hybrid_pinned(input).await?;
    let quality = build_context_quality_trace_from_pinned_hybrid(
        &report.summary.workspace_root_display,
        &report.hybrid,
        &report.pinned,
    )
    .map_err(retrieval_error_from_context)?;

    Ok(ProjectContextHybridPinnedQualityRetrievalReport {
        summary: report.summary,
        hybrid: report.hybrid,
        pinned: report.pinned,
        quality,
    })
}

fn validate_workspace_root(path: &Path) -> Result<PathBuf, ProjectContextScanError> {
    let canonical = path
        .canonicalize()
        .map_err(|_| ProjectContextScanError::InvalidWorkspace {
            diagnostics: vec![diagnostic(
                ProjectContextDiagnosticSeverity::Error,
                "project_context.workspace.invalid",
                "workspace root does not exist or cannot be accessed",
                None,
            )],
        })?;
    if !canonical.is_dir() {
        return Err(ProjectContextScanError::InvalidWorkspace {
            diagnostics: vec![diagnostic(
                ProjectContextDiagnosticSeverity::Error,
                "project_context.workspace.invalid",
                "workspace root must be a directory",
                None,
            )],
        });
    }
    Ok(canonical)
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    depth: u32,
    limits: ProjectContextLimits,
    state: &mut ScanState,
) {
    let mut entries = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .collect::<Vec<fs::DirEntry>>(),
        Err(_) => {
            if let Some(path) = relative_path(root, dir) {
                state.diagnostics.push(diagnostic(
                    ProjectContextDiagnosticSeverity::Warning,
                    "project_context.path.unreadable",
                    "path could not be read",
                    Some(path),
                ));
            }
            return;
        }
    };
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let Some(relative) = relative_path(root, &path) else {
            continue;
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                state.diagnostics.push(diagnostic(
                    ProjectContextDiagnosticSeverity::Warning,
                    "project_context.path.unreadable",
                    "path metadata could not be read",
                    Some(relative),
                ));
                continue;
            }
        };

        if file_type.is_dir() {
            if is_ignored_dir(&path) {
                state
                    .omitted
                    .push(omission(relative.as_str(), "ignored_path"));
                continue;
            }
            if depth + 1 > limits.max_depth {
                state.omitted.push(omission(relative.as_str(), "max_depth"));
                state.diagnostics.push(diagnostic(
                    ProjectContextDiagnosticSeverity::Warning,
                    "project_context.limit.max_depth",
                    "max depth reached",
                    Some(relative),
                ));
                continue;
            }
            state.directory_paths.insert(relative.clone());
            scan_dir(root, &path, depth + 1, limits, state);
            continue;
        }

        if !file_type.is_file() {
            state.omitted.push(omission(relative.as_str(), "not_file"));
            continue;
        }
        scan_file(&path, relative, limits, state);
    }
}

fn scan_file(path: &Path, relative: String, limits: ProjectContextLimits, state: &mut ScanState) {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => {
            state.diagnostics.push(diagnostic(
                ProjectContextDiagnosticSeverity::Warning,
                "project_context.path.unreadable",
                "path metadata could not be read",
                Some(relative),
            ));
            return;
        }
    };
    let size_bytes = metadata.len();

    if size_bytes > limits.max_file_bytes {
        state
            .omitted
            .push(omission(relative.as_str(), "max_file_bytes"));
        state.diagnostics.push(diagnostic(
            ProjectContextDiagnosticSeverity::Warning,
            "project_context.limit.max_file_bytes",
            "file exceeded max file bytes",
            Some(relative),
        ));
        return;
    }

    if state.used_bytes.saturating_add(size_bytes) > limits.max_total_bytes {
        state
            .omitted
            .push(omission(relative.as_str(), "max_total_bytes"));
        state.diagnostics.push(diagnostic(
            ProjectContextDiagnosticSeverity::Warning,
            "project_context.limit.max_total_bytes",
            "project scan exceeded max total bytes",
            Some(relative),
        ));
        return;
    }

    if state.files.len() >= limits.max_files as usize {
        state.omitted.push(omission(relative.as_str(), "max_files"));
        if !state.max_files_reported {
            state.max_files_reported = true;
            state.diagnostics.push(diagnostic(
                ProjectContextDiagnosticSeverity::Warning,
                "project_context.limit.max_files",
                "project scan reached max files",
                None,
            ));
        }
        return;
    }

    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            state.diagnostics.push(diagnostic(
                ProjectContextDiagnosticSeverity::Warning,
                "project_context.path.unreadable",
                "file could not be read",
                Some(relative),
            ));
            return;
        }
    };
    state.used_bytes = state.used_bytes.saturating_add(size_bytes);

    let summary = if is_binary(&bytes) {
        ProjectFileSummary::new(
            relative,
            ProjectFileKind::Binary,
            language_for_path(path),
            size_bytes,
            None,
            0,
            None,
            ProjectContextVisibility::MetadataOnly,
        )
    } else {
        let text = String::from_utf8_lossy(&bytes);
        if is_secret_like_path(path) || contains_secret_like_text(&text) {
            ProjectFileSummary::new(
                relative,
                ProjectFileKind::SecretLike,
                language_for_path(path),
                size_bytes,
                None,
                1,
                None,
                ProjectContextVisibility::MetadataOnly,
            )
        } else {
            let line_count = text.lines().count().try_into().ok();
            ProjectFileSummary::new(
                relative.clone(),
                classify_file(path, &relative),
                language_for_path(path),
                size_bytes,
                line_count,
                estimate_tokens(&text),
                Some(fnv1a64(&text)),
                ProjectContextVisibility::SummaryOnly,
            )
        }
    };

    match summary {
        Ok(summary) => state.files.push(summary),
        Err(error) => state.diagnostics.push(diagnostic(
            ProjectContextDiagnosticSeverity::Warning,
            "project_context.summary.invalid",
            &error.to_string(),
            None,
        )),
    }
}

fn directory_summaries(
    directory_paths: &HashSet<String>,
    files: &[ProjectFileSummary],
) -> Result<Vec<ProjectDirectorySummary>, ruse_context::ContextError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for path in directory_paths {
        counts.entry(path.clone()).or_default();
    }
    for file in files {
        if let Some((parent, _)) = file.path.rsplit_once('/') {
            *counts.entry(parent.to_string()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(path, file_count)| ProjectDirectorySummary::new(path, file_count))
        .collect()
}

fn relative_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    if relative.as_os_str().is_empty() {
        return Some(".".to_string());
    }
    let parts = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    Some(parts.join("/"))
}

fn is_ignored_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git" | "target" | "node_modules" | ".next" | "dist" | "build" | "coverage"
    )
}

fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(1024).any(|byte| *byte == 0)
}

fn is_secret_like_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    lower == ".env"
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.contains("secret")
        || lower.contains("credential")
}

fn contains_secret_like_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("api_key")
        || lower.contains("password")
        || lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower.contains("secret")
        || value.split_whitespace().any(|part| part.starts_with("sk-"))
}

fn classify_file(path: &Path, relative: &str) -> ProjectFileKind {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return ProjectFileKind::Unknown;
    };
    let lower = name.to_ascii_lowercase();
    if relative.starts_with("tests/") || lower.contains("test") || lower.contains("spec") {
        return ProjectFileKind::Test;
    }
    if matches!(
        lower.as_str(),
        "cargo.toml"
            | "cargo.lock"
            | "package.json"
            | "tsconfig.json"
            | "pyproject.toml"
            | "go.mod"
            | "agent.toml"
    ) || lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".json")
    {
        return ProjectFileKind::Config;
    }
    if lower.ends_with(".md") || lower.ends_with(".txt") || lower.ends_with(".rst") {
        return ProjectFileKind::Documentation;
    }
    if relative.contains("/generated/") || relative.contains("/gen/") {
        return ProjectFileKind::Generated;
    }
    if language_for_path(path).is_some() {
        return ProjectFileKind::Source;
    }
    ProjectFileKind::Unknown
}

fn language_for_path(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let language = match extension.as_str() {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "sh" | "bash" | "zsh" => "shell",
        "md" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "html" => "html",
        "css" => "css",
        _ => return None,
    };
    Some(language.to_string())
}

fn estimate_tokens(content: &str) -> u32 {
    content
        .chars()
        .count()
        .div_ceil(4)
        .max(1)
        .try_into()
        .unwrap_or(u32::MAX)
}

fn fnv1a64(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn omission(path: &str, reason: &str) -> ProjectContextOmission {
    ProjectContextOmission::new(path, reason).expect("scanner emits valid omission")
}

fn diagnostic(
    severity: ProjectContextDiagnosticSeverity,
    code: &str,
    message: &str,
    path: Option<String>,
) -> ProjectContextDiagnostic {
    ProjectContextDiagnostic::new(severity, code, message, path)
        .expect("scanner emits valid diagnostic")
}

fn scan_error_from_context(error: ruse_context::ContextError) -> ProjectContextScanError {
    ProjectContextScanError::Scan {
        diagnostics: vec![diagnostic(
            ProjectContextDiagnosticSeverity::Error,
            "project_context.summary.invalid",
            &error.to_string(),
            None,
        )],
    }
}

fn retrieval_error_from_scan(error: ProjectContextScanError) -> ProjectContextRetrievalError {
    match error {
        ProjectContextScanError::InvalidWorkspace { diagnostics } => {
            ProjectContextRetrievalError::InvalidWorkspace { diagnostics }
        }
        ProjectContextScanError::Scan { diagnostics } => {
            ProjectContextRetrievalError::Scan { diagnostics }
        }
    }
}

fn retrieval_error_from_context(
    error: ruse_context::ContextError,
) -> ProjectContextRetrievalError {
    ProjectContextRetrievalError::Retrieval {
        diagnostics: vec![diagnostic(
            ProjectContextDiagnosticSeverity::Error,
            "project_context.retrieval.invalid",
            &error.to_string(),
            None,
        )],
    }
}
