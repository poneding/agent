use std::path::{Path, PathBuf};
use std::process::Command;

use agent_model::{
    ModelEndpointConfig, ModelFallbackChain, ModelInvocationPolicy, ModelProviderCatalog,
    ModelProviderConfig, ModelProviderKind, ModelRetryPolicy,
};
use agent_types::{
    ActorRef, ClientKind, ConversationId, CorrelationId, EventId, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, SessionStarted, Timestamp, TurnId, WorkspaceId,
    WorkspaceOpened, WorkspaceTrustLevel,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, AgentRuntimeConfig, ConfigDiagnostic,
    ConfigDiagnosticSeverity, ConfigLayer, ConfigLayerKind, ConfigOverride, EffectiveRuntimeConfig,
    RuntimeModelConfig, build_effective_runtime_config, load_agent_config_file,
};

const BUILT_IN_PROVIDER: &str = "fake";
const BUILT_IN_MODEL: &str = "fake-scripted";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoSessionBootstrapInput {
    pub workspace_root: PathBuf,
    pub config_path: Option<PathBuf>,
    pub provider_override: Option<String>,
    pub model_override: Option<String>,
    pub client_kind: ClientKind,
    pub client_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepoSessionBootstrapReport {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub workspace_root_display: String,
    pub trust_level: WorkspaceTrustLevel,
    pub git: RepoGitSummary,
    pub config: RepoConfigSummary,
    pub provider: RepoProviderSummary,
    pub events: Vec<RuntimeEvent>,
    pub diagnostics: Vec<RepoSessionDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepoGitSummary {
    pub state: RepoGitState,
    pub branch: Option<String>,
    pub changed_file_count: u32,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoGitState {
    Clean,
    Dirty,
    NotRepository,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepoConfigSummary {
    pub schema_version: u32,
    pub selected_provider: String,
    pub selected_model: String,
    pub provider_ids: Vec<String>,
    pub layer_count: u32,
    pub diagnostic_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepoProviderSummary {
    pub selected_provider: String,
    pub selected_model: String,
    pub provider_kind: String,
    pub provider_count: u32,
    pub enabled_provider_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepoSessionDiagnostic {
    pub severity: RepoSessionDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoSessionDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RepoSessionBootstrapError {
    #[error("workspace root is invalid")]
    InvalidWorkspace {
        diagnostics: Vec<RepoSessionDiagnostic>,
    },
    #[error("repo session config is invalid")]
    Config {
        diagnostics: Vec<RepoSessionDiagnostic>,
    },
}

impl RepoSessionBootstrapError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidWorkspace { .. } => "repo_session.workspace.invalid",
            Self::Config { .. } => "repo_session.config.invalid",
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[RepoSessionDiagnostic] {
        match self {
            Self::InvalidWorkspace { diagnostics } | Self::Config { diagnostics } => diagnostics,
        }
    }
}

pub fn bootstrap_repo_session(
    input: RepoSessionBootstrapInput,
) -> Result<RepoSessionBootstrapReport, RepoSessionBootstrapError> {
    let workspace_root = validate_workspace_root(&input.workspace_root)
        .map_err(|diagnostics| RepoSessionBootstrapError::InvalidWorkspace { diagnostics })?;
    let (effective, mut diagnostics) = effective_config(&input)
        .map_err(|diagnostics| RepoSessionBootstrapError::Config { diagnostics })?;
    let (git, git_diagnostic) = detect_git_summary(&workspace_root);
    if let Some(diagnostic) = git_diagnostic {
        diagnostics.push(diagnostic);
    }

    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();
    let correlation_id = CorrelationId::new();
    let trust_level = WorkspaceTrustLevel::Untrusted;
    let workspace_root_display = ".".to_string();
    let provider = provider_summary(&effective);
    let config = config_summary(&effective);
    let events = bootstrap_events(
        workspace_id,
        conversation_id,
        correlation_id,
        workspace_root_display.clone(),
        trust_level.clone(),
        input.client_kind,
        input.client_version,
    );

    Ok(RepoSessionBootstrapReport {
        workspace_id,
        conversation_id,
        turn_id: None,
        workspace_root_display,
        trust_level,
        git,
        config,
        provider,
        events,
        diagnostics,
    })
}

fn validate_workspace_root(path: &Path) -> Result<PathBuf, Vec<RepoSessionDiagnostic>> {
    let canonical = path.canonicalize().map_err(|_| {
        vec![RepoSessionDiagnostic::error(
            "repo_session.workspace.invalid",
            "workspace root does not exist or cannot be accessed",
            false,
        )]
    })?;
    if !canonical.is_dir() {
        return Err(vec![RepoSessionDiagnostic::error(
            "repo_session.workspace.invalid",
            "workspace root must be a directory",
            false,
        )]);
    }
    Ok(canonical)
}

fn effective_config(
    input: &RepoSessionBootstrapInput,
) -> Result<(EffectiveRuntimeConfig, Vec<RepoSessionDiagnostic>), Vec<RepoSessionDiagnostic>> {
    let mut layers = vec![ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "built-in".to_string(),
        config: built_in_runtime_config().map_err(|error| vec![safe_config_error(error)])?,
    }];

    if let Some(path) = &input.config_path {
        let config =
            load_agent_config_file(path).map_err(|error| vec![safe_config_error(error)])?;
        layers.push(ConfigLayer {
            kind: ConfigLayerKind::User,
            source: "explicit-config".to_string(),
            config,
        });
    }

    let effective = build_effective_runtime_config(
        layers,
        ConfigOverride {
            provider: input.provider_override.clone(),
            model: input.model_override.clone(),
        },
    )
    .map_err(|error| vec![safe_config_error(error)])?;
    let diagnostics = effective
        .diagnostics
        .iter()
        .map(safe_config_diagnostic)
        .collect();
    Ok((effective, diagnostics))
}

fn built_in_runtime_config() -> Result<AgentRuntimeConfig, AgentConfigError> {
    let endpoint = ModelEndpointConfig::new(None, 30_000).map_err(config_from_model_error)?;
    let provider = ModelProviderConfig::new(
        BUILT_IN_PROVIDER,
        ModelProviderKind::Fake,
        BUILT_IN_MODEL,
        endpoint,
        None,
        true,
    )
    .map_err(config_from_model_error)?;
    let catalog = ModelProviderCatalog::new(vec![provider]).map_err(config_from_model_error)?;
    let retry = ModelRetryPolicy::new(1, Vec::new())
        .map_err(|error| config_from_message(format!("invalid retry policy: {error}")))?;
    let fallback = ModelFallbackChain::new(Vec::<String>::new())
        .map_err(|error| config_from_message(format!("invalid fallback policy: {error}")))?;
    let invocation_policy = ModelInvocationPolicy::new(30_000, retry, fallback)
        .map_err(|error| config_from_message(format!("invalid invocation policy: {error}")))?;

    Ok(AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: BUILT_IN_PROVIDER.to_string(),
            catalog,
            invocation_policy,
        },
    })
}

fn config_from_model_error(error: agent_model::ModelConfigError) -> AgentConfigError {
    config_from_message(format!("invalid built-in model config: {error}"))
}

fn config_from_message(message: String) -> AgentConfigError {
    AgentConfigError::InvalidConfig {
        diagnostics: vec![ConfigDiagnostic::error(
            "config.built_in.invalid",
            message,
            None,
        )],
    }
}

fn detect_git_summary(workspace_root: &Path) -> (RepoGitSummary, Option<RepoSessionDiagnostic>) {
    if Command::new("git").arg("--version").output().is_err() {
        return (
            RepoGitSummary {
                state: RepoGitState::Unavailable,
                branch: None,
                changed_file_count: 0,
                summary: "git unavailable".to_string(),
            },
            Some(RepoSessionDiagnostic::warning(
                "repo_session.git.unavailable",
                "git executable is unavailable; repository state was not detected",
                true,
            )),
        );
    }

    let inside = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    match inside {
        Ok(output) if output.status.success() => git_status_summary(workspace_root),
        Ok(_) => (
            RepoGitSummary {
                state: RepoGitState::NotRepository,
                branch: None,
                changed_file_count: 0,
                summary: "not a git repository".to_string(),
            },
            Some(RepoSessionDiagnostic::warning(
                "repo_session.git.not_repository",
                "workspace is not a git repository; coding session can start without git metadata",
                true,
            )),
        ),
        Err(_) => (
            RepoGitSummary {
                state: RepoGitState::Unavailable,
                branch: None,
                changed_file_count: 0,
                summary: "git status unavailable".to_string(),
            },
            Some(RepoSessionDiagnostic::warning(
                "repo_session.git.unavailable",
                "git status could not be read",
                true,
            )),
        ),
    }
}

fn git_status_summary(workspace_root: &Path) -> (RepoGitSummary, Option<RepoSessionDiagnostic>) {
    let branch = git_branch(workspace_root);
    let status = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .output();
    let Ok(output) = status else {
        return (
            RepoGitSummary {
                state: RepoGitState::Unavailable,
                branch,
                changed_file_count: 0,
                summary: "git status unavailable".to_string(),
            },
            Some(RepoSessionDiagnostic::warning(
                "repo_session.git.unavailable",
                "git status could not be read",
                true,
            )),
        );
    };
    if !output.status.success() {
        return (
            RepoGitSummary {
                state: RepoGitState::Unavailable,
                branch,
                changed_file_count: 0,
                summary: "git status unavailable".to_string(),
            },
            Some(RepoSessionDiagnostic::warning(
                "repo_session.git.unavailable",
                "git status could not be read",
                true,
            )),
        );
    }

    let changed_file_count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    if changed_file_count == 0 {
        (
            RepoGitSummary {
                state: RepoGitState::Clean,
                branch,
                changed_file_count,
                summary: "clean".to_string(),
            },
            None,
        )
    } else {
        (
            RepoGitSummary {
                state: RepoGitState::Dirty,
                branch,
                changed_file_count,
                summary: format!("dirty; changed files: {changed_file_count}"),
            },
            Some(RepoSessionDiagnostic::warning(
                "repo_session.git.dirty",
                format!("workspace has {changed_file_count} changed files"),
                true,
            )),
        )
    }
}

fn git_branch(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["branch", "--show-current"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(redact_secret_like_text(&branch))
    }
}

fn config_summary(effective: &EffectiveRuntimeConfig) -> RepoConfigSummary {
    let mut provider_ids = effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    provider_ids.sort();
    RepoConfigSummary {
        schema_version: effective.config.schema_version,
        selected_provider: effective.selected_provider.clone(),
        selected_model: effective.selected_model.clone(),
        provider_ids,
        layer_count: effective.layers.len().try_into().unwrap_or(u32::MAX),
        diagnostic_count: effective.diagnostics.len().try_into().unwrap_or(u32::MAX),
    }
}

fn provider_summary(effective: &EffectiveRuntimeConfig) -> RepoProviderSummary {
    let provider = effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .find(|provider| provider.id == effective.selected_provider);
    RepoProviderSummary {
        selected_provider: effective.selected_provider.clone(),
        selected_model: effective.selected_model.clone(),
        provider_kind: provider
            .map(|provider| model_provider_kind_label(provider.kind).to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        provider_count: effective
            .config
            .model
            .catalog
            .providers
            .len()
            .try_into()
            .unwrap_or(u32::MAX),
        enabled_provider_count: effective
            .config
            .model
            .catalog
            .providers
            .iter()
            .filter(|provider| provider.enabled)
            .count()
            .try_into()
            .unwrap_or(u32::MAX),
    }
}

fn bootstrap_events(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    correlation_id: CorrelationId,
    workspace_root_display: String,
    trust_level: WorkspaceTrustLevel,
    client_kind: ClientKind,
    client_version: String,
) -> Vec<RuntimeEvent> {
    let opened = RuntimeEventPayload::WorkspaceOpened(WorkspaceOpened {
        root_path: workspace_root_display,
        trust_level,
    });
    let session = RuntimeEventPayload::SessionStarted(SessionStarted {
        client_kind,
        client_version: redact_secret_like_text(&client_version),
    });
    let conversation = RuntimeEventPayload::ConversationStarted(agent_types::ConversationStarted {
        title: "Repo session".to_string(),
    });
    vec![opened, session, conversation]
        .into_iter()
        .map(|payload| RuntimeEvent {
            schema_version: SchemaVersion::current(),
            id: EventId::new(),
            seq: None,
            occurred_at: Timestamp::now_utc(),
            workspace_id,
            conversation_id,
            turn_id: None,
            task_id: None,
            agent_id: None,
            team_id: None,
            actor: ActorRef::system(),
            correlation_id,
            causation_id: None,
            payload,
        })
        .collect()
}

fn safe_config_error(error: AgentConfigError) -> RepoSessionDiagnostic {
    match error {
        AgentConfigError::InvalidConfig { diagnostics } => diagnostics
            .first()
            .map(safe_config_diagnostic)
            .unwrap_or_else(|| {
                RepoSessionDiagnostic::error(
                    "repo_session.config.invalid",
                    "runtime config is invalid",
                    false,
                )
            }),
        AgentConfigError::FileRead { .. } => RepoSessionDiagnostic::error(
            "repo_session.config.read_failed",
            "config file could not be read",
            false,
        ),
        AgentConfigError::Parse { .. } => RepoSessionDiagnostic::error(
            "repo_session.config.parse_failed",
            "config file could not be parsed",
            false,
        ),
        AgentConfigError::UnsupportedSchemaVersion { found, supported } => {
            RepoSessionDiagnostic::error(
                "repo_session.config.unsupported_schema",
                format!(
                    "unsupported config schema version {found}; supported version is {supported}"
                ),
                false,
            )
        }
    }
}

fn safe_config_diagnostic(diagnostic: &ConfigDiagnostic) -> RepoSessionDiagnostic {
    RepoSessionDiagnostic {
        severity: match diagnostic.severity {
            ConfigDiagnosticSeverity::Info => RepoSessionDiagnosticSeverity::Info,
            ConfigDiagnosticSeverity::Warning => RepoSessionDiagnosticSeverity::Warning,
            ConfigDiagnosticSeverity::Error => RepoSessionDiagnosticSeverity::Error,
        },
        code: redact_secret_like_text(&diagnostic.code),
        message: safe_config_message(diagnostic),
        recoverable: diagnostic.severity != ConfigDiagnosticSeverity::Error,
    }
}

fn safe_config_message(diagnostic: &ConfigDiagnostic) -> String {
    if diagnostic.severity == ConfigDiagnosticSeverity::Error {
        "runtime config is invalid".to_string()
    } else {
        redact_secret_like_text(&diagnostic.message)
    }
}

impl RepoSessionDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: RepoSessionDiagnosticSeverity::Warning,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: RepoSessionDiagnosticSeverity::Error,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }
}

fn model_provider_kind_label(kind: ModelProviderKind) -> &'static str {
    match kind {
        ModelProviderKind::Fake => "fake",
        ModelProviderKind::OpenAiCompatible => "open_ai_compatible",
        ModelProviderKind::Anthropic => "anthropic",
        ModelProviderKind::Gemini => "gemini",
    }
}

fn redact_secret_like_text(input: &str) -> String {
    let mut output = Vec::new();
    for word in input.split_whitespace() {
        let lower = word.to_ascii_lowercase();
        if lower.contains("secret")
            || lower.contains("token")
            || lower.contains("password")
            || lower.contains("api_key")
            || lower.starts_with("sk-")
            || word.contains("OPENAI_API_KEY")
        {
            output.push("[redacted]");
        } else {
            output.push(word);
        }
    }
    output.join(" ")
}
