use agent_core::{
    ObservabilityEvidenceBundle, StartupMigrationOutcome, TaskArtifact, TaskReviewSummary,
};
use agent_scheduler::{BackgroundAgentTask, TaskPriority, TaskSchedule, redact_task_text};
use agent_types::{CheckpointId, ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::{UiBridgeError, UiEventEnvelope};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiConfigSnapshot {
    pub selected_provider: String,
    pub selected_model: String,
    pub providers: Vec<UiModelProviderSummary>,
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
            diagnostics,
            platform_paths: None,
            settings: None,
        })
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
pub struct UiModelProviderSummary {
    pub id: String,
    pub kind: String,
    pub default_model: String,
    pub enabled: bool,
}

impl UiModelProviderSummary {
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        default_model: impl Into<String>,
        enabled: bool,
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
    PromptSummary(UiPromptSummary),
    ExtensionSnapshot(UiExtensionSnapshot),
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
