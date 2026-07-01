use agent_core::{
    CodingFinalReview, CommandExecutionStatus, CommandFailureClassification, CommandTestLoopReport,
    ContextCandidate, ContextCandidateOmission, ContextHybridCandidate,
    ContextHybridCandidateOmission, ContextQualityDiagnostic, ContextQualitySignal,
    ContextQualityTrace, ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity,
    ContextSemanticRetrievalSummary, DiffEvidenceState, EditDiffFlowReport,
    ObservabilityEvidenceBundle, PinnedContextItem, PinnedContextOmission,
    ProjectContextDiagnosticSeverity, ProjectContextHybridPinnedRetrievalReport,
    ProjectContextHybridRetrievalReport, ProjectContextPinnedRetrievalReport,
    ProjectContextRetrievalReport, ProjectContextSummary, ProjectFileSummary, RepoGitState,
    RepoSessionBootstrapReport, RepoSessionDiagnosticSeverity, StartupMigrationOutcome,
    TaskArtifact, TaskReviewSummary, TerminalSessionDiagnosticSeverity,
    TerminalSessionDirectoryReport, TerminalSessionStatus, TerminalSessionSummary,
    normalize_plan_todo_snapshot,
};
use agent_scheduler::{BackgroundAgentTask, TaskPriority, TaskSchedule, redact_task_text};
use agent_types::{
    CheckpointId, ClientKind, ConversationId, PlanTodoProgress, PlanTodoSnapshot, TaskId,
    TaskState, Timestamp, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use crate::approval_queue::UiApprovalQueueSnapshot;
use crate::slash::{UiSlashCommandCatalogSnapshot, UiTerminalStatusSnapshot};
use crate::{UiBridgeError, UiEventEnvelope};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiConfigSnapshot {
    pub selected_provider: String,
    pub selected_model: String,
    pub providers: Vec<UiModelProviderSummary>,
    #[serde(default)]
    pub provider_support: Vec<UiProviderSupportSummary>,
    pub diagnostics: Vec<UiConfigDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_paths: Option<UiPlatformPathsSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<UiSettingsProfileSummary>,
}

impl UiConfigSnapshot {
    pub fn new(
        selected_provider: impl Into<String>,
        selected_model: impl Into<String>,
        mut providers: Vec<UiModelProviderSummary>,
        diagnostics: Vec<UiConfigDiagnostic>,
    ) -> Result<Self, UiBridgeError> {
        let selected_provider = selected_provider.into();
        let selected_model = selected_model.into();
        validate_non_empty("selected provider", &selected_provider)?;
        validate_non_empty("selected model", &selected_model)?;
        providers.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(Self {
            selected_provider,
            selected_model,
            providers,
            provider_support: Vec::new(),
            diagnostics,
            platform_paths: None,
            settings: None,
        })
    }

    #[must_use]
    pub fn with_provider_support(
        mut self,
        mut provider_support: Vec<UiProviderSupportSummary>,
    ) -> Self {
        provider_support.sort_by(|left, right| {
            left.fallback_index
                .cmp(&right.fallback_index)
                .then_with(|| left.provider_id.cmp(&right.provider_id))
        });
        self.provider_support = provider_support;
        self
    }

    #[must_use]
    pub fn with_platform_paths(mut self, platform_paths: UiPlatformPathsSummary) -> Self {
        self.platform_paths = Some(platform_paths);
        self
    }

    #[must_use]
    pub fn with_settings(mut self, settings: UiSettingsProfileSummary) -> Self {
        self.settings = Some(settings);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPlatformPathsSummary {
    pub config_dir: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub log_dir: String,
    pub plugin_dir: String,
    pub skill_dir: String,
    pub memory_dir: String,
    pub temp_dir: String,
}

impl UiPlatformPathsSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config_dir: impl Into<String>,
        data_dir: impl Into<String>,
        cache_dir: impl Into<String>,
        log_dir: impl Into<String>,
        plugin_dir: impl Into<String>,
        skill_dir: impl Into<String>,
        memory_dir: impl Into<String>,
        temp_dir: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let summary = Self {
            config_dir: config_dir.into(),
            data_dir: data_dir.into(),
            cache_dir: cache_dir.into(),
            log_dir: log_dir.into(),
            plugin_dir: plugin_dir.into(),
            skill_dir: skill_dir.into(),
            memory_dir: memory_dir.into(),
            temp_dir: temp_dir.into(),
        };
        validate_non_empty("config dir", &summary.config_dir)?;
        validate_non_empty("data dir", &summary.data_dir)?;
        validate_non_empty("cache dir", &summary.cache_dir)?;
        validate_non_empty("log dir", &summary.log_dir)?;
        validate_non_empty("plugin dir", &summary.plugin_dir)?;
        validate_non_empty("skill dir", &summary.skill_dir)?;
        validate_non_empty("memory dir", &summary.memory_dir)?;
        validate_non_empty("temp dir", &summary.temp_dir)?;
        Ok(summary)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiSettingsProfileSummary {
    pub profile_id: String,
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    pub policy_profile: String,
    pub release_channel: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiStartupMigrationSnapshot {
    pub runtime_allowed: bool,
    pub step_count: u32,
    pub applied_count: u32,
    pub diagnostic_count: u32,
    pub domains: Vec<String>,
}

impl UiStartupMigrationSnapshot {
    pub fn from_outcome(outcome: &StartupMigrationOutcome) -> Result<Self, UiBridgeError> {
        let mut domains = outcome
            .plan
            .from_versions
            .iter()
            .map(|record| bounded_task_text(&record.domain))
            .collect::<Vec<_>>();
        domains.sort();
        Ok(Self {
            runtime_allowed: outcome.runtime_allowed,
            step_count: outcome
                .plan
                .steps
                .len()
                .try_into()
                .map_err(|_| UiBridgeError::Validation("too many migration steps".to_string()))?,
            applied_count: outcome.applied_steps.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many applied migration steps".to_string())
            })?,
            diagnostic_count: outcome.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many migration diagnostics".to_string())
            })?,
            domains,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiObservabilitySnapshot {
    pub schema_version: u32,
    pub event_count: usize,
    pub trace_count: usize,
    pub audit_count: usize,
    pub telemetry_count: usize,
    pub replayable: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiRepoSessionSnapshot {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub workspace_root_display: String,
    pub trust_level: String,
    pub git_state: String,
    pub git_branch: Option<String>,
    pub changed_file_count: u32,
    pub git_summary: String,
    pub selected_provider: String,
    pub selected_model: String,
    pub provider_kind: String,
    pub provider_count: u32,
    pub enabled_provider_count: u32,
    pub config_layer_count: u32,
    pub config_diagnostic_count: u32,
    pub diagnostic_count: u32,
    pub diagnostics: Vec<UiRepoSessionDiagnostic>,
}

impl UiRepoSessionSnapshot {
    pub fn from_report(report: &RepoSessionBootstrapReport) -> Result<Self, UiBridgeError> {
        let workspace_root_display = bounded_task_text(&report.workspace_root_display);
        validate_non_empty("workspace root", &workspace_root_display)?;
        let selected_provider = bounded_task_text(&report.provider.selected_provider);
        let selected_model = bounded_task_text(&report.provider.selected_model);
        let provider_kind = bounded_task_text(&report.provider.provider_kind);
        validate_non_empty("selected provider", &selected_provider)?;
        validate_non_empty("selected model", &selected_model)?;
        validate_non_empty("provider kind", &provider_kind)?;
        let diagnostics = report
            .diagnostics
            .iter()
            .map(UiRepoSessionDiagnostic::from_diagnostic)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            workspace_id: report.workspace_id,
            conversation_id: report.conversation_id,
            workspace_root_display,
            trust_level: format!("{:?}", report.trust_level).to_ascii_lowercase(),
            git_state: git_state_label(report.git.state).to_string(),
            git_branch: report
                .git
                .branch
                .clone()
                .map(|branch| bounded_task_text(&branch)),
            changed_file_count: report.git.changed_file_count,
            git_summary: bounded_task_text(&report.git.summary),
            selected_provider,
            selected_model,
            provider_kind,
            provider_count: report.provider.provider_count,
            enabled_provider_count: report.provider.enabled_provider_count,
            config_layer_count: report.config.layer_count,
            config_diagnostic_count: report.config.diagnostic_count,
            diagnostic_count: report.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many repo session diagnostics".to_string())
            })?,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiRepoSessionDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTerminalSessionSnapshot {
    pub sessions: Vec<UiTerminalSessionSummary>,
    pub diagnostics: Vec<UiTerminalSessionDiagnostic>,
}

impl UiTerminalSessionSnapshot {
    pub fn from_report(report: &TerminalSessionDirectoryReport) -> Result<Self, UiBridgeError> {
        let sessions = report
            .sessions
            .iter()
            .map(UiTerminalSessionSummary::from_summary)
            .collect::<Result<Vec<_>, _>>()?;
        let diagnostics = report
            .diagnostics
            .iter()
            .map(UiTerminalSessionDiagnostic::from_diagnostic)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            sessions,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTerminalSessionSummary {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub workspace_root_display: String,
    pub title: String,
    pub client_kind: String,
    pub client_version: String,
    pub provider_id: String,
    pub model_id: String,
    pub git_state: String,
    pub last_event_at: Timestamp,
    pub status: String,
    pub pending_approval_count: u32,
    pub running_command_count: u32,
    pub task_count: u32,
}

impl UiTerminalSessionSummary {
    pub fn from_summary(summary: &TerminalSessionSummary) -> Result<Self, UiBridgeError> {
        Ok(Self {
            workspace_id: summary.workspace_id,
            conversation_id: summary.conversation_id,
            workspace_root_display: bounded_non_empty(
                &summary.workspace_root_display,
                "(unknown workspace)",
            ),
            title: bounded_non_empty(&summary.title, "(untitled session)"),
            client_kind: client_kind_label(&summary.client_kind).to_string(),
            client_version: bounded_non_empty(&summary.client_version, "unknown"),
            provider_id: bounded_non_empty(&summary.provider_id, "unknown"),
            model_id: bounded_non_empty(&summary.model_id, "unknown"),
            git_state: git_state_label(summary.git_state).to_string(),
            last_event_at: summary.last_event_at,
            status: terminal_session_status_label(summary.status).to_string(),
            pending_approval_count: summary.pending_approval_count,
            running_command_count: summary.running_command_count,
            task_count: summary.task_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTerminalSessionDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiEditDiffSnapshot {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub path: String,
    pub applied: bool,
    pub diff_state: String,
    pub changed_files: Vec<String>,
    pub changed_file_count: u32,
    pub diff_line_count: u32,
    pub added_line_count: u32,
    pub removed_line_count: u32,
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub diff_preview: String,
    pub truncated: bool,
    pub redacted: bool,
    pub diagnostic_count: u32,
}

impl UiEditDiffSnapshot {
    pub fn from_report(report: &EditDiffFlowReport) -> Result<Self, UiBridgeError> {
        let path = bounded_task_text(&report.path);
        let resource = bounded_task_text(&report.preview.resource);
        let summary = bounded_task_text(&report.preview.summary);
        validate_non_empty("edit diff path", &path)?;
        validate_non_empty("edit diff resource", &resource)?;
        validate_non_empty("edit diff summary", &summary)?;
        let changed_files = report
            .changed_files
            .iter()
            .map(|file| bounded_task_text(file))
            .collect::<Vec<_>>();
        let diff_preview = bounded_diff_text(&report.diff.preview);

        Ok(Self {
            workspace_id: report.workspace_id,
            conversation_id: report.conversation_id,
            path,
            applied: report.applied,
            diff_state: diff_state_label(report.diff.state).to_string(),
            changed_files,
            changed_file_count: report.diff.changed_file_count,
            diff_line_count: report.diff.diff_line_count,
            added_line_count: report.diff.added_line_count,
            removed_line_count: report.diff.removed_line_count,
            resource,
            summary,
            content_hash: report
                .preview
                .content_hash
                .clone()
                .map(|hash| bounded_task_text(&hash)),
            diff_preview,
            truncated: report.preview.truncated || report.diff.truncated,
            redacted: report.preview.redacted || report.diff.redacted,
            diagnostic_count: report.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many edit diff diagnostics".to_string())
            })?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiCommandTestSnapshot {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub command: String,
    pub cwd: String,
    pub classify_as_test: bool,
    pub status: String,
    pub classification: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_byte_count: u64,
    pub stderr_byte_count: u64,
    pub timed_out: bool,
    pub cancellable: bool,
    pub cancellation_reason: String,
    pub observation: String,
    pub truncated: bool,
    pub redacted: bool,
    pub diagnostic_count: u32,
}

impl UiCommandTestSnapshot {
    pub fn from_report(report: &CommandTestLoopReport) -> Result<Self, UiBridgeError> {
        let command = bounded_task_text(&report.command.display);
        let cwd = bounded_task_text(&report.command.cwd);
        validate_non_empty("command test command", &command)?;
        validate_non_empty("command test cwd", &cwd)?;
        let stdout = bounded_diff_text(&report.stdout.text);
        let stderr = bounded_diff_text(&report.stderr.text);
        let observation = bounded_diff_text(&report.observation.message);
        let cancellation_reason = bounded_task_text(&report.cancellation.reason);
        validate_non_empty("command test cancellation reason", &cancellation_reason)?;

        Ok(Self {
            workspace_id: report.workspace_id,
            conversation_id: report.conversation_id,
            command,
            cwd,
            classify_as_test: report.command.classify_as_test,
            status: command_status_label(report.status).to_string(),
            classification: command_classification_label(report.classification).to_string(),
            exit_code: report.exit_code,
            stdout,
            stderr,
            stdout_byte_count: report.stdout.byte_count,
            stderr_byte_count: report.stderr.byte_count,
            timed_out: report.timed_out,
            cancellable: report.cancellation.cancellable,
            cancellation_reason,
            observation,
            truncated: report.stdout.truncated
                || report.stderr.truncated
                || report.observation.truncated,
            redacted: report.stdout.redacted
                || report.stderr.redacted
                || report.observation.redacted,
            diagnostic_count: report.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many command test diagnostics".to_string())
            })?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiFinalReviewSnapshot {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub summary: String,
    pub changed_files: Vec<String>,
    pub command_results: Vec<UiFinalReviewCommandResult>,
    pub trace_id: Option<String>,
    pub trace_event_count: u64,
    pub audit_event_count: u64,
    pub replay_reconstructed: bool,
    pub unresolved_risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub diagnostics: Vec<String>,
    pub diagnostic_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiFinalReviewCommandResult {
    pub command: String,
    pub status: String,
    pub classification: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub summary: String,
}

impl UiFinalReviewSnapshot {
    pub fn from_review(review: &CodingFinalReview) -> Result<Self, UiBridgeError> {
        let summary = bounded_diff_text(&review.summary);
        validate_non_empty("final review summary", &summary)?;
        let changed_files = review
            .changed_files
            .iter()
            .map(|file| bounded_task_text(file))
            .collect::<Vec<_>>();
        let command_results = review
            .command_results
            .iter()
            .map(|result| {
                Ok(UiFinalReviewCommandResult {
                    command: bounded_task_text(&result.command),
                    status: bounded_task_text(&result.status),
                    classification: bounded_task_text(&result.classification),
                    exit_code: result.exit_code,
                    timed_out: result.timed_out,
                    summary: bounded_diff_text(&result.summary),
                })
            })
            .collect::<Result<Vec<_>, UiBridgeError>>()?;
        let unresolved_risks = review
            .unresolved_risks
            .iter()
            .map(|risk| bounded_diff_text(risk))
            .collect::<Vec<_>>();
        let next_actions = review
            .next_actions
            .iter()
            .map(|action| bounded_diff_text(action))
            .collect::<Vec<_>>();
        let diagnostics = review
            .diagnostics
            .iter()
            .map(|diagnostic| bounded_diff_text(&diagnostic.message))
            .collect::<Vec<_>>();

        Ok(Self {
            workspace_id: review.workspace_id,
            conversation_id: review.conversation_id,
            summary,
            changed_files,
            command_results,
            trace_id: normalize_optional_string(review.trace_id.clone())
                .map(|value| bounded_task_text(&value)),
            trace_event_count: review
                .trace_event_count
                .try_into()
                .map_err(|_| UiBridgeError::Validation("too many trace events".to_string()))?,
            audit_event_count: review
                .audit_event_count
                .try_into()
                .map_err(|_| UiBridgeError::Validation("too many audit events".to_string()))?,
            replay_reconstructed: review.replay_reconstructed,
            unresolved_risks,
            next_actions,
            diagnostic_count: diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many final review diagnostics".to_string())
            })?,
            diagnostics,
        })
    }
}

impl UiRepoSessionDiagnostic {
    pub fn from_diagnostic(
        diagnostic: &agent_core::RepoSessionDiagnostic,
    ) -> Result<Self, UiBridgeError> {
        let code = bounded_task_text(&diagnostic.code);
        let message = bounded_task_text(&diagnostic.message);
        validate_non_empty("repo session diagnostic code", &code)?;
        validate_non_empty("repo session diagnostic message", &message)?;
        Ok(Self {
            severity: match diagnostic.severity {
                RepoSessionDiagnosticSeverity::Info => "info",
                RepoSessionDiagnosticSeverity::Warning => "warning",
                RepoSessionDiagnosticSeverity::Error => "error",
            }
            .to_string(),
            code,
            message,
            recoverable: diagnostic.recoverable,
        })
    }
}

impl UiTerminalSessionDiagnostic {
    pub fn from_diagnostic(
        diagnostic: &agent_core::TerminalSessionDiagnostic,
    ) -> Result<Self, UiBridgeError> {
        let code = bounded_task_text(&diagnostic.code);
        let message = bounded_task_text(&diagnostic.message);
        validate_non_empty("terminal session diagnostic code", &code)?;
        validate_non_empty("terminal session diagnostic message", &message)?;
        Ok(Self {
            severity: terminal_session_diagnostic_severity_label(diagnostic.severity).to_string(),
            code,
            message,
            recoverable: diagnostic.recoverable,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiMcpManagementSnapshot {
    pub servers: Vec<UiMcpServerManagementSummary>,
}

impl UiMcpManagementSnapshot {
    pub fn new(mut servers: Vec<UiMcpServerManagementSummary>) -> Result<Self, UiBridgeError> {
        servers.sort_by(|left, right| left.server_id.cmp(&right.server_id));
        Ok(Self { servers })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiMcpServerManagementSummary {
    pub server_id: String,
    pub display_name: String,
    pub enabled: bool,
    pub status: String,
    pub transport_kind: String,
    pub root_count: u32,
    pub sampling_available: bool,
    pub sampling_tools_supported: bool,
    pub sampling_context_supported: bool,
    pub stream_status: String,
    pub auth_status: String,
    pub extension_health: String,
    pub diagnostics: Vec<UiMcpManagementDiagnostic>,
}

impl UiMcpServerManagementSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        server_id: impl Into<String>,
        display_name: impl Into<String>,
        enabled: bool,
        status: impl Into<String>,
        transport_kind: impl Into<String>,
        root_count: u32,
        sampling_available: bool,
        sampling_tools_supported: bool,
        sampling_context_supported: bool,
        stream_status: impl Into<String>,
        auth_status: impl Into<String>,
        extension_health: impl Into<String>,
        diagnostics: Vec<UiMcpManagementDiagnostic>,
    ) -> Result<Self, UiBridgeError> {
        let summary = Self {
            server_id: bounded_task_text(&server_id.into()),
            display_name: bounded_task_text(&display_name.into()),
            enabled,
            status: bounded_task_text(&status.into()),
            transport_kind: bounded_task_text(&transport_kind.into()),
            root_count,
            sampling_available,
            sampling_tools_supported,
            sampling_context_supported,
            stream_status: bounded_task_text(&stream_status.into()),
            auth_status: bounded_task_text(&auth_status.into()),
            extension_health: bounded_task_text(&extension_health.into()),
            diagnostics: diagnostics.into_iter().take(8).collect(),
        };
        validate_non_empty("MCP server id", &summary.server_id)?;
        validate_non_empty("MCP display name", &summary.display_name)?;
        validate_non_empty("MCP status", &summary.status)?;
        validate_non_empty("MCP transport kind", &summary.transport_kind)?;
        validate_non_empty("MCP stream status", &summary.stream_status)?;
        validate_non_empty("MCP auth status", &summary.auth_status)?;
        validate_non_empty("MCP extension health", &summary.extension_health)?;
        Ok(summary)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiMcpManagementDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
}

impl UiMcpManagementDiagnostic {
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let diagnostic = Self {
            severity: bounded_task_text(&severity.into()),
            code: bounded_task_text(&code.into()),
            message: bounded_task_text(&message.into()),
        };
        validate_non_empty("MCP diagnostic severity", &diagnostic.severity)?;
        validate_non_empty("MCP diagnostic code", &diagnostic.code)?;
        validate_non_empty("MCP diagnostic message", &diagnostic.message)?;
        Ok(diagnostic)
    }
}

impl UiObservabilitySnapshot {
    pub fn from_evidence(evidence: &ObservabilityEvidenceBundle) -> Result<Self, UiBridgeError> {
        Ok(Self {
            schema_version: evidence.schema_version,
            event_count: evidence.summary.total_events,
            trace_count: evidence.summary.trace_entries,
            audit_count: evidence.summary.audit_records,
            telemetry_count: evidence.summary.telemetry_records,
            replayable: evidence.summary.replayable,
            diagnostics: evidence
                .diagnostics
                .iter()
                .map(|diagnostic| bounded_task_text(diagnostic))
                .collect(),
        })
    }
}

impl UiSettingsProfileSummary {
    pub fn new(
        profile_id: impl Into<String>,
        selected_provider: Option<String>,
        selected_model: Option<String>,
        policy_profile: impl Into<String>,
        release_channel: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let summary = Self {
            profile_id: profile_id.into(),
            selected_provider: normalize_optional_string(selected_provider),
            selected_model: normalize_optional_string(selected_model),
            policy_profile: policy_profile.into(),
            release_channel: release_channel.into(),
        };
        validate_non_empty("settings profile id", &summary.profile_id)?;
        validate_non_empty("settings policy profile", &summary.policy_profile)?;
        validate_non_empty("settings release channel", &summary.release_channel)?;
        Ok(summary)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiModelProviderCapabilities {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    pub image_input: bool,
    pub max_context_tokens: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub cost: Option<UiModelCostProfile>,
}

impl UiModelProviderCapabilities {
    #[must_use]
    pub fn new(
        streaming: bool,
        tool_calls: bool,
        structured_outputs: bool,
        image_input: bool,
        max_context_tokens: Option<u32>,
        max_output_tokens: Option<u32>,
        cost: Option<UiModelCostProfile>,
    ) -> Self {
        Self {
            streaming,
            tool_calls,
            structured_outputs,
            image_input,
            max_context_tokens,
            max_output_tokens,
            cost,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiModelCostProfile {
    pub input_per_million: Option<String>,
    pub output_per_million: Option<String>,
    pub currency: String,
}

impl UiModelCostProfile {
    pub fn new(
        input_per_million: Option<String>,
        output_per_million: Option<String>,
        currency: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let currency = currency.into();
        validate_non_empty("provider cost currency", &currency)?;
        if input_per_million
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(UiBridgeError::Validation(
                "provider cost input_per_million must not be empty".to_string(),
            ));
        }
        if output_per_million
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(UiBridgeError::Validation(
                "provider cost output_per_million must not be empty".to_string(),
            ));
        }
        Ok(Self {
            input_per_million,
            output_per_million,
            currency,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiModelProviderSummary {
    pub id: String,
    pub kind: String,
    pub default_model: String,
    pub enabled: bool,
    pub capabilities: UiModelProviderCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProviderSupportSummary {
    pub provider_id: String,
    pub kind: String,
    pub role: UiProviderSupportRole,
    pub fallback_index: Option<u32>,
    pub enabled: bool,
    pub compatible: bool,
    pub missing: Vec<String>,
}

impl UiProviderSupportSummary {
    pub fn new(
        provider_id: impl Into<String>,
        kind: impl Into<String>,
        role: UiProviderSupportRole,
        fallback_index: Option<u32>,
        enabled: bool,
        compatible: bool,
        missing: Vec<String>,
    ) -> Result<Self, UiBridgeError> {
        let provider_id = provider_id.into();
        let kind = kind.into();
        validate_non_empty("provider id", &provider_id)?;
        validate_non_empty("provider kind", &kind)?;
        for capability in &missing {
            validate_non_empty("missing capability", capability)?;
        }
        Ok(Self {
            provider_id,
            kind,
            role,
            fallback_index,
            enabled,
            compatible,
            missing,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiProviderSupportRole {
    Selected,
    Fallback,
    CatalogOnly,
}

impl UiModelProviderSummary {
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        default_model: impl Into<String>,
        enabled: bool,
        capabilities: UiModelProviderCapabilities,
    ) -> Result<Self, UiBridgeError> {
        let id = id.into();
        let kind = kind.into();
        let default_model = default_model.into();
        validate_non_empty("provider id", &id)?;
        validate_non_empty("provider kind", &kind)?;
        validate_non_empty("provider default model", &default_model)?;

        Ok(Self {
            id,
            kind,
            default_model,
            enabled,
            capabilities,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiConfigDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiConfigDiagnostic {
    pub severity: UiConfigDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field_path: Option<String>,
    pub source: Option<String>,
}

impl UiConfigDiagnostic {
    pub fn new(
        severity: UiConfigDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        source: Option<String>,
    ) -> Result<Self, UiBridgeError> {
        let code = code.into();
        let message = message.into();
        validate_non_empty("diagnostic code", &code)?;
        validate_non_empty("diagnostic message", &message)?;

        Ok(Self {
            severity,
            code,
            message,
            field_path,
            source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPromptSummary {
    pub profile_id: String,
    pub profile_version: u32,
    pub display_name: String,
    pub bundle_hash: String,
    pub estimated_tokens: u32,
    pub message_count: u32,
    pub layers: Vec<UiPromptLayerSummary>,
    pub diagnostics: Vec<UiPromptDiagnosticSummary>,
}

impl UiPromptSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile_id: impl Into<String>,
        profile_version: u32,
        display_name: impl Into<String>,
        bundle_hash: impl Into<String>,
        estimated_tokens: u32,
        message_count: u32,
        layers: Vec<UiPromptLayerSummary>,
        diagnostics: Vec<UiPromptDiagnosticSummary>,
    ) -> Result<Self, UiBridgeError> {
        let profile_id = profile_id.into();
        let display_name = display_name.into();
        let bundle_hash = bundle_hash.into();
        validate_non_empty("prompt profile id", &profile_id)?;
        validate_non_empty("prompt display name", &display_name)?;
        validate_non_empty("prompt bundle hash", &bundle_hash)?;
        if profile_version == 0 {
            return Err(UiBridgeError::Validation(
                "prompt profile version must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            profile_id,
            profile_version,
            display_name,
            bundle_hash,
            estimated_tokens,
            message_count,
            layers,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPromptLayerSummary {
    pub layer_id: String,
    pub kind: String,
    pub source_label: String,
    pub priority: i32,
    pub token_estimate: u32,
    pub visibility: String,
    pub sensitivity: String,
    pub content_preview: Option<String>,
    pub source_hash: Option<String>,
    pub content_hash: Option<String>,
}

impl UiPromptLayerSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layer_id: impl Into<String>,
        kind: impl Into<String>,
        source_label: impl Into<String>,
        priority: i32,
        token_estimate: u32,
        visibility: impl Into<String>,
        sensitivity: impl Into<String>,
        content_preview: Option<String>,
        source_hash: Option<String>,
        content_hash: Option<String>,
    ) -> Result<Self, UiBridgeError> {
        let layer_id = layer_id.into();
        let kind = kind.into();
        let source_label = source_label.into();
        let visibility = visibility.into();
        let sensitivity = sensitivity.into();
        validate_non_empty("prompt layer id", &layer_id)?;
        validate_non_empty("prompt layer kind", &kind)?;
        validate_non_empty("prompt layer source label", &source_label)?;
        validate_non_empty("prompt layer visibility", &visibility)?;
        validate_non_empty("prompt layer sensitivity", &sensitivity)?;

        let content_preview = if should_hide_prompt_preview(&visibility, &sensitivity)
            || content_preview
                .as_deref()
                .is_some_and(contains_secret_like_text)
        {
            None
        } else {
            content_preview.and_then(|preview| {
                let preview = preview.trim();
                if preview.is_empty() {
                    None
                } else {
                    Some(preview.to_string())
                }
            })
        };

        Ok(Self {
            layer_id,
            kind,
            source_label,
            priority,
            token_estimate,
            visibility,
            sensitivity,
            content_preview,
            source_hash: normalize_optional_string(source_hash),
            content_hash: normalize_optional_string(content_hash),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPromptDiagnosticSummary {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub layer_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextSnapshot {
    pub workspace_root_display: String,
    pub file_count: u32,
    pub directory_count: u32,
    pub omitted_count: u32,
    pub diagnostic_count: u32,
    pub files: Vec<UiProjectContextFileSummary>,
    pub omitted: Vec<UiProjectContextOmissionSummary>,
    pub diagnostics: Vec<UiProjectContextDiagnosticSummary>,
}

impl UiProjectContextSnapshot {
    pub fn from_summary(summary: &ProjectContextSummary) -> Result<Self, UiBridgeError> {
        let workspace_root_display = bounded_task_text(&summary.workspace_root_display);
        validate_non_empty("project context workspace root", &workspace_root_display)?;
        let files = summary
            .files
            .iter()
            .map(UiProjectContextFileSummary::from_summary)
            .collect::<Result<Vec<_>, _>>()?;
        let omitted = summary
            .omitted
            .iter()
            .map(|omission| {
                let path = bounded_task_text(&omission.path);
                let reason = bounded_task_text(&omission.reason);
                validate_non_empty("project context omission path", &path)?;
                validate_non_empty("project context omission reason", &reason)?;
                Ok(UiProjectContextOmissionSummary { path, reason })
            })
            .collect::<Result<Vec<_>, UiBridgeError>>()?;
        let diagnostics = summary
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let severity = project_context_diagnostic_severity_label(diagnostic.severity);
                let code = bounded_task_text(&diagnostic.code);
                let message = bounded_task_text(&diagnostic.message);
                validate_non_empty("project context diagnostic code", &code)?;
                validate_non_empty("project context diagnostic message", &message)?;
                Ok(UiProjectContextDiagnosticSummary {
                    severity: severity.to_string(),
                    code,
                    message,
                    path: diagnostic.path.clone().map(|path| bounded_task_text(&path)),
                })
            })
            .collect::<Result<Vec<_>, UiBridgeError>>()?;

        Ok(Self {
            workspace_root_display,
            file_count: summary.files.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context files".to_string())
            })?,
            directory_count: summary.directories.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context directories".to_string())
            })?,
            omitted_count: summary.omitted.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context omissions".to_string())
            })?,
            diagnostic_count: summary.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context diagnostics".to_string())
            })?,
            files,
            omitted,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextFileSummary {
    pub path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub visibility: String,
}

impl UiProjectContextFileSummary {
    fn from_summary(summary: &ProjectFileSummary) -> Result<Self, UiBridgeError> {
        let path = bounded_task_text(&summary.path);
        validate_non_empty("project context file path", &path)?;
        Ok(Self {
            path,
            kind: summary.kind.as_str().to_string(),
            language: summary
                .language
                .clone()
                .map(|language| bounded_task_text(&language)),
            size_bytes: summary.size_bytes,
            line_count: summary.line_count,
            token_estimate: summary.token_estimate,
            visibility: summary.visibility.as_str().to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextOmissionSummary {
    pub path: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextDiagnosticSummary {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextRetrievalSnapshot {
    pub workspace_root_display: String,
    pub task: String,
    pub max_candidates: u32,
    pub budget_tokens: u32,
    pub used_tokens: u32,
    pub selected_count: u32,
    pub omitted_count: u32,
    pub diagnostic_count: u32,
    pub selected: Vec<UiProjectContextRetrievalCandidateSummary>,
    pub omitted: Vec<UiProjectContextRetrievalOmissionSummary>,
    pub diagnostics: Vec<UiProjectContextRetrievalDiagnosticSummary>,
}

impl UiProjectContextRetrievalSnapshot {
    pub fn from_report(report: &ProjectContextRetrievalReport) -> Result<Self, UiBridgeError> {
        let workspace_root_display = bounded_task_text(&report.summary.workspace_root_display);
        validate_non_empty(
            "project context retrieval workspace root",
            &workspace_root_display,
        )?;
        let task = bounded_task_text(&report.retrieval.query.task);
        validate_non_empty("project context retrieval task", &task)?;
        let selected = report
            .retrieval
            .selected
            .iter()
            .map(UiProjectContextRetrievalCandidateSummary::from_candidate)
            .collect::<Result<Vec<_>, _>>()?;
        let omitted = report
            .retrieval
            .omitted
            .iter()
            .map(UiProjectContextRetrievalOmissionSummary::from_omission)
            .collect::<Result<Vec<_>, _>>()?;
        let diagnostics = report
            .retrieval
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let severity =
                    context_retrieval_diagnostic_severity_label(diagnostic.severity).to_string();
                let code = bounded_task_text(&diagnostic.code);
                let message = bounded_task_text(&diagnostic.message);
                validate_non_empty("project context retrieval diagnostic code", &code)?;
                validate_non_empty("project context retrieval diagnostic message", &message)?;
                Ok(UiProjectContextRetrievalDiagnosticSummary {
                    severity,
                    code,
                    message,
                    path: diagnostic.path.clone().map(|path| bounded_task_text(&path)),
                })
            })
            .collect::<Result<Vec<_>, UiBridgeError>>()?;

        Ok(Self {
            workspace_root_display,
            task,
            max_candidates: report.retrieval.query.max_candidates,
            budget_tokens: report.retrieval.query.budget_tokens,
            used_tokens: report.retrieval.used_tokens,
            selected_count: report.retrieval.selected.len().try_into().map_err(|_| {
                UiBridgeError::Validation(
                    "too many project context retrieval selections".to_string(),
                )
            })?,
            omitted_count: report.retrieval.omitted.len().try_into().map_err(|_| {
                UiBridgeError::Validation(
                    "too many project context retrieval omissions".to_string(),
                )
            })?,
            diagnostic_count: report.retrieval.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation(
                    "too many project context retrieval diagnostics".to_string(),
                )
            })?,
            selected,
            omitted,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextRetrievalCandidateSummary {
    pub path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub visibility: String,
    pub score: u32,
    pub reasons: Vec<String>,
}

impl UiProjectContextRetrievalCandidateSummary {
    fn from_candidate(candidate: &ContextCandidate) -> Result<Self, UiBridgeError> {
        let path = bounded_task_text(&candidate.path);
        validate_non_empty("project context retrieval candidate path", &path)?;
        let reasons = candidate
            .reasons
            .iter()
            .map(|reason| reason.as_str().to_string())
            .collect::<Vec<_>>();
        if reasons.is_empty() {
            return Err(UiBridgeError::Validation(
                "project context retrieval candidate reasons cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            path,
            kind: candidate.kind.as_str().to_string(),
            language: candidate
                .language
                .clone()
                .map(|language| bounded_task_text(&language)),
            size_bytes: candidate.size_bytes,
            line_count: candidate.line_count,
            token_estimate: candidate.token_estimate,
            visibility: candidate.visibility.as_str().to_string(),
            score: candidate.score,
            reasons,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextRetrievalOmissionSummary {
    pub path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub visibility: String,
    pub score: u32,
    pub reasons: Vec<String>,
    pub reason: String,
}

impl UiProjectContextRetrievalOmissionSummary {
    fn from_omission(omission: &ContextCandidateOmission) -> Result<Self, UiBridgeError> {
        let candidate =
            UiProjectContextRetrievalCandidateSummary::from_candidate(&omission.candidate)?;
        Ok(Self {
            path: candidate.path,
            kind: candidate.kind,
            language: candidate.language,
            size_bytes: candidate.size_bytes,
            line_count: candidate.line_count,
            token_estimate: candidate.token_estimate,
            visibility: candidate.visibility,
            score: candidate.score,
            reasons: candidate.reasons,
            reason: omission.reason.as_str().to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextRetrievalDiagnosticSummary {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

fn retrieval_diagnostic_summary(
    diagnostic: &ContextRetrievalDiagnostic,
) -> Result<UiProjectContextRetrievalDiagnosticSummary, UiBridgeError> {
    let severity = context_retrieval_diagnostic_severity_label(diagnostic.severity).to_string();
    let code = bounded_task_text(&diagnostic.code);
    let message = bounded_task_text(&diagnostic.message);
    validate_non_empty("project context retrieval diagnostic code", &code)?;
    validate_non_empty("project context retrieval diagnostic message", &message)?;
    Ok(UiProjectContextRetrievalDiagnosticSummary {
        severity,
        code,
        message,
        path: diagnostic.path.clone().map(|path| bounded_task_text(&path)),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiProjectContextRetrievalScore(f64);

impl Eq for UiProjectContextRetrievalScore {}

impl UiProjectContextRetrievalScore {
    pub fn new(score: f64) -> Result<Self, UiBridgeError> {
        if !score.is_finite() {
            return Err(UiBridgeError::Validation(
                "project context retrieval score must be finite".to_string(),
            ));
        }
        Ok(Self((score * 1_000.0).round() / 1_000.0))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextHybridRetrievalSnapshot {
    pub workspace_root_display: String,
    pub task: String,
    pub max_candidates: u32,
    pub budget_tokens: u32,
    pub used_tokens: u32,
    pub selected_count: u32,
    pub omitted_count: u32,
    pub diagnostic_count: u32,
    pub semantic: UiProjectContextSemanticRetrievalSummary,
    pub selected: Vec<UiProjectContextHybridCandidateSummary>,
    pub omitted: Vec<UiProjectContextHybridOmissionSummary>,
    pub diagnostics: Vec<UiProjectContextRetrievalDiagnosticSummary>,
}

impl UiProjectContextHybridRetrievalSnapshot {
    pub fn from_report(
        report: &ProjectContextHybridRetrievalReport,
    ) -> Result<Self, UiBridgeError> {
        let workspace_root_display = bounded_task_text(&report.summary.workspace_root_display);
        validate_non_empty(
            "project context hybrid retrieval workspace root",
            &workspace_root_display,
        )?;
        let task = bounded_task_text(&report.hybrid.query.task);
        validate_non_empty("project context hybrid retrieval task", &task)?;
        let selected = report
            .hybrid
            .selected
            .iter()
            .map(UiProjectContextHybridCandidateSummary::from_candidate)
            .collect::<Result<Vec<_>, _>>()?;
        let omitted = report
            .hybrid
            .omitted
            .iter()
            .map(UiProjectContextHybridOmissionSummary::from_omission)
            .collect::<Result<Vec<_>, _>>()?;
        let diagnostics = report
            .hybrid
            .diagnostics
            .iter()
            .map(retrieval_diagnostic_summary)
            .collect::<Result<Vec<_>, UiBridgeError>>()?;

        Ok(Self {
            workspace_root_display,
            task,
            max_candidates: report.hybrid.query.max_candidates,
            budget_tokens: report.hybrid.query.budget_tokens,
            used_tokens: report.hybrid.used_tokens,
            selected_count: report.hybrid.selected.len().try_into().map_err(|_| {
                UiBridgeError::Validation(
                    "too many project context hybrid retrieval selections".to_string(),
                )
            })?,
            omitted_count: report.hybrid.omitted.len().try_into().map_err(|_| {
                UiBridgeError::Validation(
                    "too many project context hybrid retrieval omissions".to_string(),
                )
            })?,
            diagnostic_count: report.hybrid.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation(
                    "too many project context hybrid retrieval diagnostics".to_string(),
                )
            })?,
            semantic: UiProjectContextSemanticRetrievalSummary::from_summary(
                &report.hybrid.semantic,
            ),
            selected,
            omitted,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextSemanticRetrievalSummary {
    pub status: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub dimensions: Option<usize>,
    pub document_count: u32,
    pub embedded_document_count: u32,
}

impl UiProjectContextSemanticRetrievalSummary {
    fn from_summary(summary: &ContextSemanticRetrievalSummary) -> Self {
        Self {
            status: summary.status.as_str().to_string(),
            provider_id: summary
                .provider_id
                .clone()
                .map(|provider| bounded_task_text(&provider)),
            model_id: summary
                .model_id
                .clone()
                .map(|model| bounded_task_text(&model)),
            dimensions: summary.dimensions,
            document_count: summary.document_count,
            embedded_document_count: summary.embedded_document_count,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextHybridCandidateSummary {
    pub path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub visibility: String,
    pub score: u32,
    pub lexical_score: UiProjectContextRetrievalScore,
    pub semantic_score: Option<UiProjectContextRetrievalScore>,
    pub combined_score: UiProjectContextRetrievalScore,
    pub reasons: Vec<String>,
}

impl UiProjectContextHybridCandidateSummary {
    fn from_candidate(candidate: &ContextHybridCandidate) -> Result<Self, UiBridgeError> {
        let base = UiProjectContextRetrievalCandidateSummary::from_candidate(&candidate.candidate)?;
        Ok(Self {
            path: base.path,
            kind: base.kind,
            language: base.language,
            size_bytes: base.size_bytes,
            line_count: base.line_count,
            token_estimate: base.token_estimate,
            visibility: base.visibility,
            score: base.score,
            lexical_score: UiProjectContextRetrievalScore::new(candidate.lexical_score)?,
            semantic_score: candidate
                .semantic_score
                .map(UiProjectContextRetrievalScore::new)
                .transpose()?,
            combined_score: UiProjectContextRetrievalScore::new(candidate.combined_score)?,
            reasons: base.reasons,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextHybridOmissionSummary {
    pub path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<u32>,
    pub token_estimate: u32,
    pub visibility: String,
    pub score: u32,
    pub lexical_score: UiProjectContextRetrievalScore,
    pub semantic_score: Option<UiProjectContextRetrievalScore>,
    pub combined_score: UiProjectContextRetrievalScore,
    pub reasons: Vec<String>,
    pub reason: String,
}

impl UiProjectContextHybridOmissionSummary {
    fn from_omission(omission: &ContextHybridCandidateOmission) -> Result<Self, UiBridgeError> {
        let candidate =
            UiProjectContextHybridCandidateSummary::from_candidate(&omission.candidate)?;
        Ok(Self {
            path: candidate.path,
            kind: candidate.kind,
            language: candidate.language,
            size_bytes: candidate.size_bytes,
            line_count: candidate.line_count,
            token_estimate: candidate.token_estimate,
            visibility: candidate.visibility,
            score: candidate.score,
            lexical_score: candidate.lexical_score,
            semantic_score: candidate.semantic_score,
            combined_score: candidate.combined_score,
            reasons: candidate.reasons,
            reason: omission.reason.as_str().to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextPinnedRetrievalSnapshot {
    pub workspace_root_display: String,
    pub task: String,
    pub retrieval_kind: String,
    pub max_candidates: u32,
    pub retrieval_budget_tokens: u32,
    pub context_budget_tokens: u32,
    pub reserved_response_tokens: u32,
    pub available_tokens: u32,
    pub used_tokens: u32,
    pub included_count: u32,
    pub omitted_count: u32,
    pub diagnostic_count: u32,
    pub retrieval_included: bool,
    pub retrieval_omitted_reason: Option<String>,
    pub included: Vec<UiPinnedContextItemSummary>,
    pub omitted: Vec<UiPinnedContextOmissionSummary>,
    pub diagnostics: Vec<UiProjectContextRetrievalDiagnosticSummary>,
}

impl UiProjectContextPinnedRetrievalSnapshot {
    pub fn from_retrieval_report(
        report: &ProjectContextPinnedRetrievalReport,
    ) -> Result<Self, UiBridgeError> {
        Self::from_parts(
            &report.summary.workspace_root_display,
            &report.retrieval.query.task,
            "lexical",
            report.retrieval.query.max_candidates,
            report.retrieval.query.budget_tokens,
            &report.pinned,
        )
    }

    pub fn from_hybrid_report(
        report: &ProjectContextHybridPinnedRetrievalReport,
    ) -> Result<Self, UiBridgeError> {
        Self::from_parts(
            &report.summary.workspace_root_display,
            &report.hybrid.query.task,
            "hybrid",
            report.hybrid.query.max_candidates,
            report.hybrid.query.budget_tokens,
            &report.pinned,
        )
    }

    fn from_parts(
        workspace_root_display: &str,
        task: &str,
        retrieval_kind: &str,
        max_candidates: u32,
        retrieval_budget_tokens: u32,
        pinned: &agent_core::PinnedProjectContextBudgetReport,
    ) -> Result<Self, UiBridgeError> {
        let workspace_root_display = bounded_task_text(workspace_root_display);
        validate_non_empty(
            "project context pinned retrieval workspace root",
            &workspace_root_display,
        )?;
        let task = bounded_task_text(task);
        validate_non_empty("project context pinned retrieval task", &task)?;
        let retrieval_kind = bounded_task_text(retrieval_kind);
        validate_non_empty("project context pinned retrieval kind", &retrieval_kind)?;
        let included = pinned
            .included
            .iter()
            .map(UiPinnedContextItemSummary::from_item)
            .collect::<Result<Vec<_>, _>>()?;
        let omitted = pinned
            .omitted
            .iter()
            .map(UiPinnedContextOmissionSummary::from_omission)
            .collect::<Result<Vec<_>, _>>()?;
        let diagnostics = pinned
            .diagnostics
            .iter()
            .map(retrieval_diagnostic_summary)
            .collect::<Result<Vec<_>, UiBridgeError>>()?;

        Ok(Self {
            workspace_root_display,
            task,
            retrieval_kind,
            max_candidates,
            retrieval_budget_tokens,
            context_budget_tokens: pinned.budget.max_tokens,
            reserved_response_tokens: pinned.budget.reserved_response_tokens,
            available_tokens: pinned.available_tokens,
            used_tokens: pinned.used_tokens,
            included_count: pinned.included.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context pinned inclusions".to_string())
            })?,
            omitted_count: pinned.omitted.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context pinned omissions".to_string())
            })?,
            diagnostic_count: pinned.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many project context pinned diagnostics".to_string())
            })?,
            retrieval_included: pinned.retrieval_included,
            retrieval_omitted_reason: pinned
                .retrieval_omitted_reason
                .clone()
                .map(|reason| bounded_task_text(&reason)),
            included,
            omitted,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPinnedContextItemSummary {
    pub path: String,
    pub kind: String,
    pub language: Option<String>,
    pub visibility: String,
    pub token_estimate: u32,
    pub priority: String,
    pub source: String,
    pub reason: String,
}

impl UiPinnedContextItemSummary {
    fn from_item(item: &PinnedContextItem) -> Result<Self, UiBridgeError> {
        let path = bounded_task_text(&item.path);
        let reason = bounded_task_text(&item.reason);
        validate_non_empty("pinned context item path", &path)?;
        validate_non_empty("pinned context item reason", &reason)?;
        Ok(Self {
            path,
            kind: item.kind.as_str().to_string(),
            language: item
                .language
                .clone()
                .map(|language| bounded_task_text(&language)),
            visibility: item.visibility.as_str().to_string(),
            token_estimate: item.token_estimate,
            priority: item.priority.as_str().to_string(),
            source: item.source.as_str().to_string(),
            reason,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPinnedContextOmissionSummary {
    pub path: String,
    pub priority: String,
    pub reason: String,
    pub message: String,
}

impl UiPinnedContextOmissionSummary {
    fn from_omission(omission: &PinnedContextOmission) -> Result<Self, UiBridgeError> {
        let path = bounded_task_text(&omission.path);
        let message = bounded_task_text(&omission.message);
        validate_non_empty("pinned context omission path", &path)?;
        validate_non_empty("pinned context omission message", &message)?;
        Ok(Self {
            path,
            priority: omission.priority.as_str().to_string(),
            reason: omission.reason.as_str().to_string(),
            message,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiProjectContextQualityTraceSnapshot {
    pub workspace_root_display: String,
    pub task: String,
    pub retrieval_kind: String,
    pub selected_count: u32,
    pub omitted_count: u32,
    pub pinned_included_count: u32,
    pub pinned_omitted_count: u32,
    pub used_tokens: u32,
    pub available_tokens: u32,
    pub budget_pressure: String,
    pub signal_count: u32,
    pub diagnostic_count: u32,
    pub signals: Vec<UiContextQualitySignalSummary>,
    pub diagnostics: Vec<UiContextQualityDiagnosticSummary>,
}

impl UiProjectContextQualityTraceSnapshot {
    pub fn from_trace(trace: &ContextQualityTrace) -> Result<Self, UiBridgeError> {
        let workspace_root_display = bounded_task_text(&trace.workspace_root_display);
        validate_non_empty("context quality workspace root", &workspace_root_display)?;
        let task = bounded_task_text(&trace.task);
        validate_non_empty("context quality task", &task)?;
        let signals = trace
            .signals
            .iter()
            .map(UiContextQualitySignalSummary::from_signal)
            .collect::<Result<Vec<_>, _>>()?;
        let diagnostics = trace
            .diagnostics
            .iter()
            .map(UiContextQualityDiagnosticSummary::from_diagnostic)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            workspace_root_display,
            task,
            retrieval_kind: trace.retrieval_kind.as_str().to_string(),
            selected_count: trace.selected_count,
            omitted_count: trace.omitted_count,
            pinned_included_count: trace.pinned_included_count,
            pinned_omitted_count: trace.pinned_omitted_count,
            used_tokens: trace.used_tokens,
            available_tokens: trace.available_tokens,
            budget_pressure: trace.budget_pressure.as_str().to_string(),
            signal_count: trace.signals.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many context quality signals".to_string())
            })?,
            diagnostic_count: trace.diagnostics.len().try_into().map_err(|_| {
                UiBridgeError::Validation("too many context quality diagnostics".to_string())
            })?,
            signals,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiContextQualitySignalSummary {
    pub path: Option<String>,
    pub kind: String,
    pub weight: u32,
    pub message: String,
}

impl UiContextQualitySignalSummary {
    fn from_signal(signal: &ContextQualitySignal) -> Result<Self, UiBridgeError> {
        let path = signal.path.clone().map(|path| bounded_task_text(&path));
        let message = bounded_task_text(&signal.message);
        validate_non_empty("context quality signal message", &message)?;
        Ok(Self {
            path,
            kind: signal.kind.as_str().to_string(),
            weight: signal.weight,
            message,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiContextQualityDiagnosticSummary {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

impl UiContextQualityDiagnosticSummary {
    fn from_diagnostic(diagnostic: &ContextQualityDiagnostic) -> Result<Self, UiBridgeError> {
        let code = bounded_task_text(&diagnostic.code);
        let message = bounded_task_text(&diagnostic.message);
        validate_non_empty("context quality diagnostic code", &code)?;
        validate_non_empty("context quality diagnostic message", &message)?;
        Ok(Self {
            severity: diagnostic.severity.as_str().to_string(),
            code,
            message,
            path: diagnostic.path.clone().map(|path| bounded_task_text(&path)),
        })
    }
}

impl UiPromptDiagnosticSummary {
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        layer_id: Option<String>,
    ) -> Result<Self, UiBridgeError> {
        let severity = severity.into();
        let code = code.into();
        let message = message.into();
        validate_non_empty("prompt diagnostic severity", &severity)?;
        validate_non_empty("prompt diagnostic code", &code)?;
        validate_non_empty("prompt diagnostic message", &message)?;

        Ok(Self {
            severity,
            code,
            message,
            layer_id: normalize_optional_string(layer_id),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionSnapshot {
    pub extensions: Vec<UiExtensionSummary>,
    pub capability_reviews: Vec<UiExtensionCapabilityReview>,
    pub diagnostics: Vec<UiExtensionDiagnostic>,
}

impl UiExtensionSnapshot {
    pub fn new(
        mut extensions: Vec<UiExtensionSummary>,
        capability_reviews: Vec<UiExtensionCapabilityReview>,
        diagnostics: Vec<UiExtensionDiagnostic>,
    ) -> Result<Self, UiBridgeError> {
        extensions.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(Self {
            extensions,
            capability_reviews,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionSummary {
    pub id: String,
    pub kind: String,
    pub enabled: bool,
    pub health: String,
    pub version: Option<String>,
    pub source_label: String,
    pub capability_count: u32,
}

impl UiExtensionSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        enabled: bool,
        health: impl Into<String>,
        version: Option<String>,
        source_label: impl Into<String>,
        capability_count: u32,
    ) -> Result<Self, UiBridgeError> {
        let id = id.into();
        let kind = kind.into();
        let health = health.into();
        let source_label = source_label.into();
        validate_non_empty("extension id", &id)?;
        validate_non_empty("extension kind", &kind)?;
        validate_non_empty("extension health", &health)?;
        validate_non_empty("extension source label", &source_label)?;

        Ok(Self {
            id,
            kind,
            enabled,
            health: redact_secret_like_text(&health),
            version: normalize_optional_string(version)
                .map(|value| redact_secret_like_text(&value)),
            source_label: redact_secret_like_text(&source_label),
            capability_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionCapabilityReview {
    pub extension_id: String,
    pub capability_kind: String,
    pub resource: String,
    pub reason: String,
}

impl UiExtensionCapabilityReview {
    pub fn new(
        extension_id: impl Into<String>,
        capability_kind: impl Into<String>,
        resource: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let extension_id = extension_id.into();
        let capability_kind = capability_kind.into();
        let resource = resource.into();
        let reason = reason.into();
        validate_non_empty("extension review id", &extension_id)?;
        validate_non_empty("extension review capability kind", &capability_kind)?;
        validate_non_empty("extension review resource", &resource)?;
        validate_non_empty("extension review reason", &reason)?;

        Ok(Self {
            extension_id,
            capability_kind: redact_secret_like_text(&capability_kind),
            resource: redact_secret_like_text(&resource),
            reason: redact_secret_like_text(&reason),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub extension_id: Option<String>,
    pub truncated: bool,
}

impl UiExtensionDiagnostic {
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        extension_id: Option<String>,
        truncated: bool,
    ) -> Result<Self, UiBridgeError> {
        let severity = severity.into();
        let code = code.into();
        let message = message.into();
        validate_non_empty("extension diagnostic severity", &severity)?;
        validate_non_empty("extension diagnostic code", &code)?;
        validate_non_empty("extension diagnostic message", &message)?;

        Ok(Self {
            severity: redact_secret_like_text(&severity),
            code,
            message: redact_secret_like_text(&message),
            extension_id: normalize_optional_string(extension_id),
            truncated,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPlanTodoSnapshot {
    pub snapshot: PlanTodoSnapshot,
    pub progress: PlanTodoProgress,
    pub errors: Vec<UiPlanTodoCommandError>,
}

impl UiPlanTodoSnapshot {
    pub fn from_snapshot(
        snapshot: PlanTodoSnapshot,
        errors: Vec<UiPlanTodoCommandError>,
    ) -> Result<Self, UiBridgeError> {
        let (mut snapshot, progress) = normalize_plan_todo_snapshot(snapshot)
            .map_err(|error| UiBridgeError::Validation(error.to_string()))?;
        snapshot.items.sort_by_key(|item| item.item_id.clone());

        Ok(Self {
            snapshot,
            progress,
            errors,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPlanTodoCommandError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl UiPlanTodoCommandError {
    pub fn recoverable(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        Self::new(code, message, true)
    }

    pub fn fatal(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        Self::new(code, message, false)
    }

    fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        recoverable: bool,
    ) -> Result<Self, UiBridgeError> {
        let code = bounded_task_text(&code.into());
        let message = bounded_task_text(&message.into());
        validate_non_empty("plan/todo command error code", &code)?;
        validate_non_empty("plan/todo command error message", &message)?;

        Ok(Self {
            code,
            message,
            recoverable,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskSnapshot {
    pub tasks: Vec<UiTaskSummary>,
    pub errors: Vec<UiTaskCommandError>,
}

impl UiTaskSnapshot {
    pub fn new(
        mut tasks: Vec<UiTaskSummary>,
        errors: Vec<UiTaskCommandError>,
    ) -> Result<Self, UiBridgeError> {
        tasks.sort_by_key(|task| task.task_id);

        Ok(Self { tasks, errors })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskSummary {
    pub task_id: TaskId,
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub schedule: UiTaskScheduleSummary,
    pub prompt_summary: String,
    pub latest_run_summary: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl UiTaskSummary {
    pub fn from_task(task: &BackgroundAgentTask) -> Result<Self, UiBridgeError> {
        let prompt_summary = bounded_task_text(&task.prompt);
        validate_non_empty("task prompt summary", &prompt_summary)?;

        Ok(Self {
            task_id: task.task_id,
            workspace_id: task.workspace_id,
            conversation_id: task.conversation_id,
            state: task.state,
            priority: task.priority,
            schedule: UiTaskScheduleSummary::from_schedule(&task.schedule)?,
            prompt_summary,
            latest_run_summary: task
                .latest_run_summary
                .clone()
                .map(|summary| bounded_task_text(&summary)),
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskScheduleSummary {
    pub kind: String,
    pub label: String,
}

impl UiTaskScheduleSummary {
    pub fn from_schedule(schedule: &TaskSchedule) -> Result<Self, UiBridgeError> {
        let kind = match schedule {
            TaskSchedule::OneShot { .. } => "one_shot",
            TaskSchedule::Cron { .. } => "cron",
        }
        .to_string();
        let label = bounded_task_text(&schedule.label());
        validate_non_empty("task schedule label", &label)?;

        Ok(Self { kind, label })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskCommandError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl UiTaskCommandError {
    pub fn recoverable(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        Self::new(code, message, true)
    }

    pub fn fatal(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        Self::new(code, message, false)
    }

    fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        recoverable: bool,
    ) -> Result<Self, UiBridgeError> {
        let code = code.into();
        let message = bounded_task_text(&message.into());
        validate_non_empty("task command error code", &code)?;
        validate_non_empty("task command error message", &message)?;

        Ok(Self {
            code,
            message,
            recoverable,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskReviewSnapshot {
    pub reviews: Vec<UiTaskReviewSummary>,
    pub errors: Vec<UiTaskCommandError>,
}

impl UiTaskReviewSnapshot {
    pub fn new(
        mut reviews: Vec<UiTaskReviewSummary>,
        errors: Vec<UiTaskCommandError>,
    ) -> Result<Self, UiBridgeError> {
        reviews.sort_by_key(|review| review.task_id);
        Ok(Self { reviews, errors })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskReviewSummary {
    pub task_id: TaskId,
    pub final_state: TaskState,
    pub trace_id: Option<String>,
    pub checkpoint_id: Option<CheckpointId>,
    pub artifact_ids: Vec<String>,
    pub artifact_count: u32,
    pub summary: String,
    pub artifacts: Vec<UiTaskReviewSummaryArtifact>,
    pub diagnostics: Vec<String>,
}

impl UiTaskReviewSummary {
    pub fn from_review(
        review: &TaskReviewSummary,
        artifacts: &[TaskArtifact],
    ) -> Result<Self, UiBridgeError> {
        let summary = bounded_task_text(&review.summary);
        validate_non_empty("task review summary", &summary)?;

        let mut ui_artifacts = Vec::new();
        let mut diagnostics = review
            .diagnostics
            .iter()
            .map(|diagnostic| bounded_task_text(diagnostic))
            .collect::<Vec<_>>();

        for artifact_id in &review.artifact_ids {
            if let Some(artifact) = artifacts
                .iter()
                .find(|artifact| artifact.artifact_id == *artifact_id)
            {
                ui_artifacts.push(UiTaskReviewSummaryArtifact::from_artifact(artifact)?);
            } else {
                diagnostics.push(bounded_task_text(&format!(
                    "missing artifact metadata: {artifact_id}"
                )));
            }
        }

        Ok(Self {
            task_id: review.task_id,
            final_state: review.final_state,
            trace_id: normalize_optional_string(review.trace_id.clone())
                .map(|value| bounded_task_text(&value)),
            checkpoint_id: review.checkpoint_id,
            artifact_ids: review.artifact_ids.clone(),
            artifact_count: ui_artifacts
                .len()
                .try_into()
                .map_err(|_| UiBridgeError::Validation("too many task artifacts".to_string()))?,
            summary,
            artifacts: ui_artifacts,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTaskReviewSummaryArtifact {
    pub artifact_id: String,
    pub kind: String,
    pub path: Option<String>,
    pub content_hash: Option<String>,
    pub summary: String,
    pub missing: bool,
    pub diagnostics: Vec<String>,
}

impl UiTaskReviewSummaryArtifact {
    pub fn from_artifact(artifact: &TaskArtifact) -> Result<Self, UiBridgeError> {
        let artifact_id = artifact.artifact_id.trim().to_string();
        let kind = artifact.kind.as_str().to_string();
        let summary = bounded_task_text(&artifact.summary);
        validate_non_empty("task artifact id", &artifact_id)?;
        validate_non_empty("task artifact kind", &kind)?;
        validate_non_empty("task artifact summary", &summary)?;

        Ok(Self {
            artifact_id,
            kind,
            path: normalize_optional_string(artifact.path.clone())
                .map(|value| bounded_task_text(&value)),
            content_hash: normalize_optional_string(artifact.content_hash.clone())
                .map(|value| bounded_task_text(&value)),
            summary,
            missing: artifact.missing,
            diagnostics: artifact
                .diagnostics
                .iter()
                .map(|diagnostic| bounded_task_text(diagnostic))
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiJsonFrame {
    Event(UiEventEnvelope),
    ConfigSnapshot(UiConfigSnapshot),
    RepoSessionSnapshot(UiRepoSessionSnapshot),
    TerminalSessionSnapshot(UiTerminalSessionSnapshot),
    ApprovalQueueSnapshot(UiApprovalQueueSnapshot),
    SlashCommandCatalogSnapshot(UiSlashCommandCatalogSnapshot),
    TerminalStatusSnapshot(UiTerminalStatusSnapshot),
    EditDiffSnapshot(UiEditDiffSnapshot),
    CommandTestSnapshot(UiCommandTestSnapshot),
    FinalReviewSnapshot(UiFinalReviewSnapshot),
    PromptSummary(UiPromptSummary),
    ProjectContextSnapshot(UiProjectContextSnapshot),
    ProjectContextRetrievalSnapshot(UiProjectContextRetrievalSnapshot),
    ProjectContextHybridRetrievalSnapshot(UiProjectContextHybridRetrievalSnapshot),
    ProjectContextPinnedRetrievalSnapshot(UiProjectContextPinnedRetrievalSnapshot),
    ProjectContextQualityTraceSnapshot(UiProjectContextQualityTraceSnapshot),
    ExtensionSnapshot(UiExtensionSnapshot),
    PlanTodoSnapshot(UiPlanTodoSnapshot),
    TaskSnapshot(UiTaskSnapshot),
    TaskReviewSnapshot(UiTaskReviewSnapshot),
    StartupMigrationSnapshot(UiStartupMigrationSnapshot),
    ObservabilitySnapshot(UiObservabilitySnapshot),
    ModelSelection {
        provider: String,
        model: String,
    },
    ConfigSummary {
        selected_provider: String,
        selected_model: String,
        provider_ids: Vec<String>,
        layer_count: u64,
        diagnostic_count: u64,
    },
    ConfigDiagnostic {
        severity: String,
        code: String,
        message: String,
        field_path: Option<String>,
        source: Option<String>,
    },
    Done {
        event_count: u64,
    },
    Error {
        code: String,
        message: String,
        recoverable: bool,
    },
}

fn git_state_label(state: RepoGitState) -> &'static str {
    match state {
        RepoGitState::Clean => "clean",
        RepoGitState::Dirty => "dirty",
        RepoGitState::NotRepository => "not_repository",
        RepoGitState::Unavailable => "unavailable",
    }
}

fn client_kind_label(kind: &ClientKind) -> &'static str {
    match kind {
        ClientKind::Cli => "cli",
        ClientKind::Tui => "tui",
        ClientKind::Desktop => "desktop",
        ClientKind::Api => "api",
        ClientKind::Test => "test",
    }
}

fn terminal_session_status_label(status: TerminalSessionStatus) -> &'static str {
    match status {
        TerminalSessionStatus::Idle => "idle",
        TerminalSessionStatus::HasTasks => "has_tasks",
        TerminalSessionStatus::RunningCommand => "running_command",
        TerminalSessionStatus::WaitingApproval => "waiting_approval",
        TerminalSessionStatus::Failed => "failed",
    }
}

fn terminal_session_diagnostic_severity_label(
    severity: TerminalSessionDiagnosticSeverity,
) -> &'static str {
    match severity {
        TerminalSessionDiagnosticSeverity::Info => "info",
        TerminalSessionDiagnosticSeverity::Warning => "warning",
        TerminalSessionDiagnosticSeverity::Error => "error",
    }
}

fn diff_state_label(state: DiffEvidenceState) -> &'static str {
    match state {
        DiffEvidenceState::Preview => "preview",
        DiffEvidenceState::Applied => "applied",
        DiffEvidenceState::Unavailable => "unavailable",
    }
}

fn command_status_label(status: CommandExecutionStatus) -> &'static str {
    match status {
        CommandExecutionStatus::Succeeded => "succeeded",
        CommandExecutionStatus::Failed => "failed",
        CommandExecutionStatus::TimedOut => "timed_out",
        CommandExecutionStatus::WaitingApproval => "waiting_approval",
        CommandExecutionStatus::Denied => "denied",
        CommandExecutionStatus::ToolFailed => "tool_failed",
    }
}

fn command_classification_label(classification: CommandFailureClassification) -> &'static str {
    match classification {
        CommandFailureClassification::None => "none",
        CommandFailureClassification::TestFailure => "test_failure",
        CommandFailureClassification::CommandFailure => "command_failure",
        CommandFailureClassification::Timeout => "timeout",
        CommandFailureClassification::PolicyDenied => "policy_denied",
        CommandFailureClassification::WaitingApproval => "waiting_approval",
        CommandFailureClassification::ToolFailure => "tool_failure",
        CommandFailureClassification::InvalidInput => "invalid_input",
    }
}

fn project_context_diagnostic_severity_label(
    severity: ProjectContextDiagnosticSeverity,
) -> &'static str {
    match severity {
        ProjectContextDiagnosticSeverity::Info => "info",
        ProjectContextDiagnosticSeverity::Warning => "warning",
        ProjectContextDiagnosticSeverity::Error => "error",
    }
}

fn context_retrieval_diagnostic_severity_label(
    severity: ContextRetrievalDiagnosticSeverity,
) -> &'static str {
    match severity {
        ContextRetrievalDiagnosticSeverity::Info => "info",
        ContextRetrievalDiagnosticSeverity::Warning => "warning",
        ContextRetrievalDiagnosticSeverity::Error => "error",
    }
}

pub fn encode_json_frame(frame: &UiJsonFrame) -> Result<String, UiBridgeError> {
    let mut line = serde_json::to_string(frame)?;
    line.push('\n');
    Ok(line)
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), UiBridgeError> {
    if value.trim().is_empty() {
        return Err(UiBridgeError::Validation(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(())
}

fn bounded_non_empty(value: &str, fallback: &str) -> String {
    let bounded = bounded_task_text(value);
    if bounded.trim().is_empty() {
        fallback.to_string()
    } else {
        bounded
    }
}

fn should_hide_prompt_preview(visibility: &str, sensitivity: &str) -> bool {
    visibility.eq_ignore_ascii_case("hidden") || sensitivity.eq_ignore_ascii_case("secret_like")
}

fn contains_secret_like_text(value: &str) -> bool {
    value.split_whitespace().any(is_secret_like_token)
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            if is_secret_like_token(part) {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn bounded_task_text(value: &str) -> String {
    let redacted = redact_task_text(value);
    let redacted = redacted.trim();
    if redacted.chars().count() > 120 {
        redacted.chars().take(120).collect()
    } else {
        redacted.to_string()
    }
}

fn bounded_diff_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    let redacted = redacted.trim();
    if redacted.chars().count() > 4_000 {
        redacted.chars().take(4_000).collect()
    } else {
        redacted.to_string()
    }
}

fn is_secret_like_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}
