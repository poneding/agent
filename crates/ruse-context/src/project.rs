use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    ContextError, PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity,
    PromptLayerSource, PromptLayerVisibility,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectContextLimits {
    pub max_files: u32,
    pub max_depth: u32,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
}

impl ProjectContextLimits {
    pub fn new(
        max_files: u32,
        max_depth: u32,
        max_file_bytes: u64,
        max_total_bytes: u64,
    ) -> Result<Self, ContextError> {
        if max_files == 0 {
            return Err(ContextError::InvalidProjectContext(
                "max_files must be greater than zero".to_string(),
            ));
        }
        if max_depth == 0 {
            return Err(ContextError::InvalidProjectContext(
                "max_depth must be greater than zero".to_string(),
            ));
        }
        if max_file_bytes == 0 {
            return Err(ContextError::InvalidProjectContext(
                "max_file_bytes must be greater than zero".to_string(),
            ));
        }
        if max_total_bytes == 0 {
            return Err(ContextError::InvalidProjectContext(
                "max_total_bytes must be greater than zero".to_string(),
            ));
        }
        Ok(Self {
            max_files,
            max_depth,
            max_file_bytes,
            max_total_bytes,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectFileKind {
    Source,
    Test,
    Config,
    Documentation,
    Generated,
    Binary,
    SecretLike,
    Unknown,
}

impl ProjectFileKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Test => "test",
            Self::Config => "config",
            Self::Documentation => "documentation",
            Self::Generated => "generated",
            Self::Binary => "binary",
            Self::SecretLike => "secret_like",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectContextVisibility {
    MetadataOnly,
    SummaryOnly,
    Full,
}

impl ProjectContextVisibility {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::SummaryOnly => "summary_only",
            Self::Full => "full",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectFileSummary {
    pub path: String,
    pub kind: ProjectFileKind,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub content_hash: Option<String>,
    pub visibility: ProjectContextVisibility,
}

impl ProjectFileSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: impl Into<String>,
        kind: ProjectFileKind,
        language: Option<String>,
        size_bytes: u64,
        line_count: Option<u32>,
        token_estimate: u32,
        content_hash: Option<String>,
        visibility: ProjectContextVisibility,
    ) -> Result<Self, ContextError> {
        let path = path.into();
        validate_non_empty("file path", &path)?;
        let language = normalize_optional(language, "language")?;
        let content_hash = normalize_optional(content_hash, "content hash")?;
        Ok(Self {
            path,
            kind,
            language,
            size_bytes,
            line_count,
            token_estimate,
            content_hash,
            visibility,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectDirectorySummary {
    pub path: String,
    pub file_count: u32,
}

impl ProjectDirectorySummary {
    pub fn new(path: impl Into<String>, file_count: u32) -> Result<Self, ContextError> {
        let path = path.into();
        validate_non_empty("directory path", &path)?;
        Ok(Self { path, file_count })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectContextOmission {
    pub path: String,
    pub reason: String,
}

impl ProjectContextOmission {
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Result<Self, ContextError> {
        let path = path.into();
        let reason = reason.into();
        validate_non_empty("omission path", &path)?;
        validate_non_empty("omission reason", &reason)?;
        Ok(Self { path, reason })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectContextDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl ProjectContextDiagnosticSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectContextDiagnostic {
    pub severity: ProjectContextDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

impl ProjectContextDiagnostic {
    pub fn new(
        severity: ProjectContextDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<String>,
    ) -> Result<Self, ContextError> {
        let code = code.into();
        let message = message.into();
        validate_non_empty("diagnostic code", &code)?;
        validate_non_empty("diagnostic message", &message)?;
        let path = normalize_optional(path, "diagnostic path")?;
        Ok(Self {
            severity,
            code,
            message,
            path,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectContextSummary {
    pub workspace_root_display: String,
    pub files: Vec<ProjectFileSummary>,
    pub directories: Vec<ProjectDirectorySummary>,
    pub omitted: Vec<ProjectContextOmission>,
    pub diagnostics: Vec<ProjectContextDiagnostic>,
    pub limits: ProjectContextLimits,
}

impl ProjectContextSummary {
    pub fn new(
        workspace_root_display: impl Into<String>,
        mut files: Vec<ProjectFileSummary>,
        mut directories: Vec<ProjectDirectorySummary>,
        mut omitted: Vec<ProjectContextOmission>,
        mut diagnostics: Vec<ProjectContextDiagnostic>,
        limits: ProjectContextLimits,
    ) -> Result<Self, ContextError> {
        let workspace_root_display = workspace_root_display.into();
        validate_non_empty("workspace root display", &workspace_root_display)?;
        ensure_unique_paths("file", files.iter().map(|file| file.path.as_str()))?;
        ensure_unique_paths(
            "directory",
            directories.iter().map(|directory| directory.path.as_str()),
        )?;
        files.sort_by(|left, right| left.path.cmp(&right.path));
        directories.sort_by(|left, right| left.path.cmp(&right.path));
        omitted.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.reason.cmp(&right.reason))
        });
        diagnostics.sort_by(|left, right| {
            left.code
                .cmp(&right.code)
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.message.cmp(&right.message))
        });

        Ok(Self {
            workspace_root_display,
            files,
            directories,
            omitted,
            diagnostics,
            limits,
        })
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
                path: self.workspace_root_display.clone(),
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
            "Project context for {}: {} files, {} directories, {} omitted.",
            self.workspace_root_display,
            self.files.len(),
            self.directories.len(),
            self.omitted.len()
        )];
        if !self.files.is_empty() {
            lines.push("Files:".to_string());
            for file in self.files.iter().take(80) {
                let language = file
                    .language
                    .as_deref()
                    .map(|language| format!(" {language}"))
                    .unwrap_or_default();
                lines.push(format!(
                    "- {} [{}{}; {}; {} bytes]",
                    file.path,
                    file.kind.as_str(),
                    language,
                    file.visibility.as_str(),
                    file.size_bytes
                ));
            }
        }
        if !self.omitted.is_empty() {
            lines.push("Omitted:".to_string());
            for omission in self.omitted.iter().take(40) {
                lines.push(format!("- {}: {}", omission.path, omission.reason));
            }
        }
        lines.join("\n")
    }
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), ContextError> {
    if value.trim().is_empty() {
        return Err(ContextError::InvalidProjectContext(format!(
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
                Err(ContextError::InvalidProjectContext(format!(
                    "{label} cannot be empty"
                )))
            } else {
                Ok(value)
            }
        })
        .transpose()
}

fn ensure_unique_paths<'a>(
    label: &str,
    paths: impl Iterator<Item = &'a str>,
) -> Result<(), ContextError> {
    let mut seen = HashSet::new();
    for path in paths {
        if !seen.insert(path) {
            return Err(ContextError::InvalidProjectContext(format!(
                "duplicate {label} path {path}"
            )));
        }
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
