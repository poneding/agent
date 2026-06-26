use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;

use crate::output::CliOutput;
use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, AgentRuntimeConfig, ConfigDiagnostic,
    ConfigDiagnosticSeverity, ConfigLayer, ConfigLayerKind, ConfigOverride, ConfiguredTurnInput,
    ConfiguredTurnOutcome, ConfiguredTurnRunner, EffectiveRuntimeConfig, RuntimeIds,
    RuntimeModelConfig,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelConfigError, ModelEndpointConfig,
    ModelFallbackChain, ModelInvocationPolicy, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequestRequirements, ModelRetryPolicy,
    RegisteredModelProvider,
};
use agent_storage::EventStore;
use agent_types::{ConversationId, CorrelationId, TurnId, WorkspaceId};
use agent_ui_bridge::{
    UiBridgeError, UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot, UiJsonFrame,
    UiModelProviderSummary, encode_json_frame, normalize_runtime_event,
};

const DEFAULT_MODEL_PROVIDER: &str = "fake";
const DEFAULT_MODEL_NAME: &str = "fake-scripted";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Version,
    RuntimeBanner,
    BatchPrompt { prompt: String },
    TtyShell,
    LocalSummary,
    TraceSummary,
    ConfigCheck,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelSelection {
    pub provider: String,
    pub model: String,
}

impl ModelSelection {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Result<Self, CliError> {
        let provider = provider.into();
        let model = model.into();
        let provider = provider.trim();
        let model = model.trim();

        if provider.is_empty() {
            return Err(CliError::EmptyModelProvider);
        }
        if model.is_empty() {
            return Err(CliError::EmptyModelName);
        }

        let endpoint = ModelEndpointConfig::new(None, 30_000)
            .map_err(|error| CliError::InvalidModelSelection(error.to_string()))?;
        ModelProviderConfig::new(
            provider,
            ModelProviderKind::Fake,
            model,
            endpoint,
            None,
            true,
        )
        .map_err(map_model_config_error)?;

        Ok(Self {
            provider: provider.to_string(),
            model: model.to_string(),
        })
    }
}

impl Default for ModelSelection {
    fn default() -> Self {
        Self {
            provider: DEFAULT_MODEL_PROVIDER.to_string(),
            model: DEFAULT_MODEL_NAME.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliInvocation {
    pub command: CliCommand,
    pub output_mode: OutputMode,
    pub model_selection: ModelSelection,
    pub config_path: Option<PathBuf>,
    pub config_override: ConfigOverride,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CliError {
    #[error("unknown argument: {0}")]
    UnknownArgument(String),
    #[error("missing value for argument: {0}")]
    MissingValue(String),
    #[error("model provider cannot be empty")]
    EmptyModelProvider,
    #[error("model name cannot be empty")]
    EmptyModelName,
    #[error("invalid model provider: {0}")]
    InvalidModelProvider(String),
    #[error("invalid model selection: {0}")]
    InvalidModelSelection(String),
    #[error("config check failed: {message}")]
    ConfigCheckFailed {
        message: String,
        diagnostics: Vec<ConfigDiagnostic>,
    },
    #[error("prompt runtime failed: {message}")]
    PromptRuntimeFailed { message: String },
}

pub fn parse_cli_command<I, S>(args: I) -> Result<CliCommand, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let Some(first) = args.next() else {
        return Ok(CliCommand::RuntimeBanner);
    };

    match first.as_str() {
        "--version" | "-V" => Ok(CliCommand::Version),
        "--tty" => Ok(CliCommand::TtyShell),
        "--local-summary" => Ok(CliCommand::LocalSummary),
        "--trace-summary" => Ok(CliCommand::TraceSummary),
        "--config-check" => Ok(CliCommand::ConfigCheck),
        "--prompt" | "-p" => {
            let prompt = args
                .next()
                .ok_or_else(|| CliError::MissingValue(first.clone()))?;
            Ok(CliCommand::BatchPrompt { prompt })
        }
        _ => Err(CliError::UnknownArgument(first)),
    }
}

pub fn parse_cli_invocation<I, S>(args: I) -> Result<CliInvocation, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut output_mode = OutputMode::Text;
    let mut command_args = Vec::new();
    let mut provider = None;
    let mut model = None;
    let mut config_path = None;
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => output_mode = OutputMode::Json,
            "--provider" => provider = Some(take_flag_value(&mut args, &arg)?),
            "--model" => model = Some(take_flag_value(&mut args, &arg)?),
            "--config" => config_path = Some(PathBuf::from(take_flag_value(&mut args, &arg)?)),
            _ => command_args.push(arg),
        }
    }

    let config_override = ConfigOverride {
        provider: provider.clone(),
        model: model.clone(),
    };

    Ok(CliInvocation {
        command: parse_cli_command(command_args)?,
        output_mode,
        model_selection: ModelSelection::new(
            provider.unwrap_or_else(|| DEFAULT_MODEL_PROVIDER.to_string()),
            model.unwrap_or_else(|| DEFAULT_MODEL_NAME.to_string()),
        )?,
        config_path,
        config_override,
    })
}

#[must_use]
pub fn render_cli_command(command: CliCommand) -> CliOutput {
    match command {
        CliCommand::Version => {
            CliOutput::exit(format!("agent-cli {}\n", env!("CARGO_PKG_VERSION")))
        }
        CliCommand::RuntimeBanner => CliOutput::exit(format!("{}\n", agent_core::runtime_banner())),
        CliCommand::BatchPrompt { prompt } => {
            render_prompt_runtime(prompt, None, ConfigOverride::default())
        }
        CliCommand::TtyShell => CliOutput::keep_running("agent-cli tty shell ready\n"),
        CliCommand::LocalSummary => match agent_core::run_local_coding_fixture() {
            Ok(run) => CliOutput::exit(render_local_coding_summary(&run.outcome)),
            Err(error) => CliOutput::exit(format!("local coding summary failed: {error}\n")),
        },
        CliCommand::TraceSummary => match render_local_trace_summary() {
            Ok(output) => CliOutput::exit(output),
            Err(error) => CliOutput::exit(format!("trace summary failed: {error}\n")),
        },
        CliCommand::ConfigCheck => render_text_config_check(None, ConfigOverride::default()),
    }
}

#[must_use]
pub fn render_cli_invocation(invocation: CliInvocation) -> CliOutput {
    match invocation.output_mode {
        OutputMode::Text => match invocation.command {
            CliCommand::ConfigCheck => {
                render_text_config_check(invocation.config_path, invocation.config_override)
            }
            CliCommand::BatchPrompt { prompt } => {
                render_prompt_runtime(prompt, invocation.config_path, invocation.config_override)
            }
            command => render_cli_command(command),
        },
        OutputMode::Json => render_json_invocation(invocation),
    }
}

fn render_json_invocation(invocation: CliInvocation) -> CliOutput {
    if invocation.command == CliCommand::ConfigCheck {
        return render_json_config_check(invocation.config_path, invocation.config_override);
    }

    let mut stdout = render_model_selection_frame(&invocation.model_selection);

    match invocation.command {
        CliCommand::LocalSummary => match agent_core::run_local_coding_fixture() {
            Ok(run) => {
                let mut event_count = 0;

                for event in run.events {
                    let ui_event = normalize_runtime_event(&event);
                    stdout.push_str(&json_frame_line(UiJsonFrame::Event(ui_event)));
                    event_count += 1;
                }

                stdout.push_str(&json_frame_line(UiJsonFrame::Done { event_count }));
                CliOutput::exit(stdout)
            }
            Err(error) => {
                stdout.push_str(&json_frame_line(UiJsonFrame::Error {
                    code: "local_summary_failed".to_string(),
                    message: error.to_string(),
                    recoverable: false,
                }));
                CliOutput::exit(stdout)
            }
        },
        CliCommand::BatchPrompt { prompt } => render_json_prompt_runtime(
            stdout,
            prompt,
            invocation.config_path,
            invocation.config_override,
        ),
        other => {
            stdout.push_str(&json_frame_line(UiJsonFrame::Error {
                code: "unsupported_json_command".to_string(),
                message: format!("{other:?} does not support JSON output"),
                recoverable: false,
            }));
            CliOutput::exit(stdout)
        }
    }
}

fn render_prompt_runtime(
    prompt: String,
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> CliOutput {
    match run_prompt_runtime(prompt, config_path, config_override) {
        Ok(final_message) => CliOutput::exit(format!("{final_message}\n")),
        Err(error) => CliOutput::failure(format!("{error}\n"), 2),
    }
}

fn run_prompt_runtime(
    prompt: String,
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> Result<String, CliError> {
    Ok(run_prompt_runtime_outcome(prompt, config_path, config_override)?.final_message)
}

fn run_prompt_runtime_outcome(
    prompt: String,
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> Result<ConfiguredTurnOutcome, CliError> {
    let safe_prompt = redact_secret_like_text(&prompt);
    let effective = evaluate_config_check(config_path, config_override)?;
    let registry = fake_provider_registry(&effective, &safe_prompt)?;
    let input = ConfiguredTurnInput {
        ids: runtime_ids(),
        user_message: safe_prompt,
        effective_config: effective,
        requirements: prompt_requirements(),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|error| CliError::PromptRuntimeFailed {
            message: error.to_string(),
        })?;
    runtime
        .block_on(ConfiguredTurnRunner::run(input, &registry))
        .map_err(|error| CliError::PromptRuntimeFailed {
            message: error.to_string(),
        })
}

fn render_json_prompt_runtime(
    mut stdout: String,
    prompt: String,
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> CliOutput {
    match run_prompt_runtime_outcome(prompt, config_path, config_override) {
        Ok(outcome) => {
            let mut event_count = 0;
            for event in outcome.events {
                stdout.push_str(&json_frame_line(UiJsonFrame::Event(
                    normalize_runtime_event(&event),
                )));
                event_count += 1;
            }
            stdout.push_str(&json_frame_line(UiJsonFrame::Done { event_count }));
            CliOutput::exit(stdout)
        }
        Err(error) => {
            stdout.push_str(&json_frame_line(UiJsonFrame::Error {
                code: "prompt_runtime_failed".to_string(),
                message: error.to_string(),
                recoverable: false,
            }));
            stdout.push_str(&json_frame_line(UiJsonFrame::Done { event_count: 0 }));
            CliOutput::failure(stdout, 2)
        }
    }
}

fn fake_provider_registry(
    effective: &EffectiveRuntimeConfig,
    prompt: &str,
) -> Result<ModelProviderRegistry, CliError> {
    let providers = effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .filter(|provider| provider.enabled && provider.kind == ModelProviderKind::Fake)
        .map(|provider| {
            RegisteredModelProvider::new(
                provider.clone(),
                fake_capabilities(),
                Arc::new(FakeModelProvider::new(FakeModelScript::text(
                    &fake_response(prompt),
                ))),
            )
            .map_err(|error| CliError::PromptRuntimeFailed {
                message: error.to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    ModelProviderRegistry::new(providers).map_err(|error| CliError::PromptRuntimeFailed {
        message: error.to_string(),
    })
}

fn fake_response(prompt: &str) -> String {
    format!("fake response: {}", redact_secret_like_text(prompt))
}

fn redact_secret_like_text(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("api_key")
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

fn fake_capabilities() -> ModelCapabilities {
    ModelCapabilities {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        max_context_tokens: Some(4096),
    }
}

fn prompt_requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        min_context_tokens: Some(1024),
    }
}

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn render_text_config_check(
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> CliOutput {
    match evaluate_config_check(config_path, config_override) {
        Ok(effective) => CliOutput::exit(render_config_summary_text(&effective)),
        Err(error) => CliOutput::failure(format!("{error}\n"), 2),
    }
}

fn render_json_config_check(
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> CliOutput {
    match evaluate_config_check(config_path, config_override) {
        Ok(effective) => {
            let snapshot = match ui_config_snapshot(&effective) {
                Ok(snapshot) => snapshot,
                Err(error) => return render_json_config_check_error(error),
            };
            let mut stdout = String::new();
            for diagnostic in &effective.diagnostics {
                stdout.push_str(&json_frame_line(config_diagnostic_frame(diagnostic)));
            }
            stdout.push_str(&json_frame_line(UiJsonFrame::ConfigSnapshot(snapshot)));
            stdout.push_str(&json_frame_line(config_summary_frame(&effective)));
            stdout.push_str(&json_frame_line(UiJsonFrame::Done { event_count: 0 }));
            CliOutput::exit(stdout)
        }
        Err(error) => render_json_config_check_error(error),
    }
}

fn render_json_config_check_error(error: CliError) -> CliOutput {
    let mut stdout = String::new();
    if let CliError::ConfigCheckFailed {
        message,
        diagnostics,
    } = &error
    {
        for diagnostic in diagnostics {
            stdout.push_str(&json_frame_line(config_diagnostic_frame(diagnostic)));
        }
        stdout.push_str(&json_frame_line(UiJsonFrame::Error {
            code: "config_check_failed".to_string(),
            message: message.clone(),
            recoverable: false,
        }));
    }
    stdout.push_str(&json_frame_line(UiJsonFrame::Done { event_count: 0 }));
    CliOutput::failure(stdout, 2)
}

fn evaluate_config_check(
    config_path: Option<PathBuf>,
    config_override: ConfigOverride,
) -> Result<EffectiveRuntimeConfig, CliError> {
    let mut layers = vec![ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "built-in".to_string(),
        config: built_in_runtime_config()?,
    }];

    if let Some(path) = config_path {
        let source = path.display().to_string();
        let config = agent_core::load_agent_config_file(&path).map_err(map_agent_config_error)?;
        layers.push(ConfigLayer {
            kind: ConfigLayerKind::User,
            source,
            config,
        });
    }

    agent_core::build_effective_runtime_config(layers, config_override)
        .map_err(map_agent_config_error)
}

fn built_in_runtime_config() -> Result<AgentRuntimeConfig, CliError> {
    let endpoint = ModelEndpointConfig::new(None, 30_000)
        .map_err(|error| CliError::InvalidModelSelection(error.to_string()))?;
    let provider = ModelProviderConfig::new(
        DEFAULT_MODEL_PROVIDER,
        ModelProviderKind::Fake,
        DEFAULT_MODEL_NAME,
        endpoint,
        None,
        true,
    )
    .map_err(map_model_config_error)?;
    let catalog = ModelProviderCatalog::new(vec![provider]).map_err(map_model_config_error)?;
    let retry = ModelRetryPolicy::new(1, Vec::new())
        .map_err(|error| CliError::InvalidModelSelection(error.to_string()))?;
    let fallback = ModelFallbackChain::new(Vec::<String>::new())
        .map_err(|error| CliError::InvalidModelSelection(error.to_string()))?;
    let invocation_policy = ModelInvocationPolicy::new(30_000, retry, fallback)
        .map_err(|error| CliError::InvalidModelSelection(error.to_string()))?;

    Ok(AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: DEFAULT_MODEL_PROVIDER.to_string(),
            catalog,
            invocation_policy,
        },
    })
}

fn render_config_summary_text(effective: &EffectiveRuntimeConfig) -> String {
    let provider_ids = config_provider_ids(effective).join(", ");
    format!(
        "Config: valid\nSelected provider: {}\nSelected model: {}\nProviders: {}\nLayers: {}\nDiagnostics: {}\n",
        effective.selected_provider,
        effective.selected_model,
        provider_ids,
        effective.layers.len(),
        effective.diagnostics.len()
    )
}

fn config_summary_frame(effective: &EffectiveRuntimeConfig) -> UiJsonFrame {
    UiJsonFrame::ConfigSummary {
        selected_provider: effective.selected_provider.clone(),
        selected_model: effective.selected_model.clone(),
        provider_ids: config_provider_ids(effective),
        layer_count: effective.layers.len() as u64,
        diagnostic_count: effective.diagnostics.len() as u64,
    }
}

fn config_provider_ids(effective: &EffectiveRuntimeConfig) -> Vec<String> {
    let mut provider_ids = effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    provider_ids.sort();
    provider_ids
}

fn ui_config_snapshot(effective: &EffectiveRuntimeConfig) -> Result<UiConfigSnapshot, CliError> {
    let providers = effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .map(|provider| {
            UiModelProviderSummary::new(
                provider.id.clone(),
                model_provider_kind_label(provider.kind),
                provider.default_model.clone(),
                provider.enabled,
            )
            .map_err(map_ui_bridge_error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let diagnostics = effective
        .diagnostics
        .iter()
        .map(ui_config_diagnostic)
        .collect::<Result<Vec<_>, _>>()?;

    UiConfigSnapshot::new(
        effective.selected_provider.clone(),
        effective.selected_model.clone(),
        providers,
        diagnostics,
    )
    .map_err(map_ui_bridge_error)
}

fn ui_config_diagnostic(diagnostic: &ConfigDiagnostic) -> Result<UiConfigDiagnostic, CliError> {
    UiConfigDiagnostic::new(
        ui_config_diagnostic_severity(diagnostic.severity),
        diagnostic.code.clone(),
        diagnostic.message.clone(),
        diagnostic.field_path.clone(),
        diagnostic.source.clone(),
    )
    .map_err(map_ui_bridge_error)
}

fn ui_config_diagnostic_severity(severity: ConfigDiagnosticSeverity) -> UiConfigDiagnosticSeverity {
    match severity {
        ConfigDiagnosticSeverity::Info => UiConfigDiagnosticSeverity::Info,
        ConfigDiagnosticSeverity::Warning => UiConfigDiagnosticSeverity::Warning,
        ConfigDiagnosticSeverity::Error => UiConfigDiagnosticSeverity::Error,
    }
}

fn model_provider_kind_label(kind: ModelProviderKind) -> &'static str {
    match kind {
        ModelProviderKind::Fake => "fake",
        ModelProviderKind::OpenAiCompatible => "open_ai_compatible",
    }
}

fn config_diagnostic_frame(diagnostic: &ConfigDiagnostic) -> UiJsonFrame {
    UiJsonFrame::ConfigDiagnostic {
        severity: format!("{:?}", diagnostic.severity).to_ascii_lowercase(),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
        field_path: diagnostic.field_path.clone(),
        source: diagnostic.source.clone(),
    }
}

fn map_agent_config_error(error: AgentConfigError) -> CliError {
    let diagnostics = match &error {
        AgentConfigError::InvalidConfig { diagnostics } => diagnostics.clone(),
        _ => Vec::new(),
    };
    CliError::ConfigCheckFailed {
        message: format_agent_config_error(error),
        diagnostics,
    }
}

fn map_ui_bridge_error(error: UiBridgeError) -> CliError {
    CliError::ConfigCheckFailed {
        message: error.to_string(),
        diagnostics: Vec::new(),
    }
}

fn format_agent_config_error(error: AgentConfigError) -> String {
    match error {
        AgentConfigError::InvalidConfig { diagnostics } => diagnostics
            .into_iter()
            .map(|diagnostic| {
                let field = diagnostic
                    .field_path
                    .map(|field_path| format!(" at {field_path}"))
                    .unwrap_or_default();
                format!("{}{}: {}", diagnostic.code, field, diagnostic.message)
            })
            .collect::<Vec<_>>()
            .join("; "),
        other => other.to_string(),
    }
}

fn render_model_selection_frame(selection: &ModelSelection) -> String {
    if selection == &ModelSelection::default() {
        String::new()
    } else {
        json_frame_line(UiJsonFrame::ModelSelection {
            provider: selection.provider.clone(),
            model: selection.model.clone(),
        })
    }
}

fn take_flag_value<I>(args: &mut I, flag: &str) -> Result<String, CliError>
where
    I: Iterator<Item = String>,
{
    let value = args
        .next()
        .ok_or_else(|| CliError::MissingValue(flag.to_string()))?;
    if value.starts_with("--") {
        Err(CliError::MissingValue(flag.to_string()))
    } else {
        Ok(value)
    }
}

fn map_model_config_error(error: ModelConfigError) -> CliError {
    match error {
        ModelConfigError::EmptyProviderId => CliError::EmptyModelProvider,
        ModelConfigError::InvalidProviderId { id } => CliError::InvalidModelProvider(id),
        ModelConfigError::EmptyDefaultModel { .. } => CliError::EmptyModelName,
        other => CliError::InvalidModelSelection(other.to_string()),
    }
}

fn json_frame_line(frame: UiJsonFrame) -> String {
    match encode_json_frame(&frame) {
        Ok(line) => line,
        Err(error) => {
            format!(
                "{{\"type\":\"error\",\"data\":{{\"code\":\"json_encode_failed\",\"message\":\"{}\",\"recoverable\":false}}}}\n",
                error.to_string().replace('"', "\\\"")
            )
        }
    }
}

fn render_local_trace_summary() -> Result<String, String> {
    let run = agent_core::run_local_coding_fixture().map_err(|error| error.to_string())?;
    let conversation_id = run
        .events
        .first()
        .map(|event| event.conversation_id)
        .ok_or_else(|| "local coding fixture produced no events".to_string())?;
    let mut store = agent_storage::InMemoryEventStore::default();
    for event in run.events {
        store.append(event).map_err(|error| error.to_string())?;
    }
    let entries = agent_core::build_trace(
        &store,
        agent_core::TraceQuery {
            conversation_id,
            include_tools: true,
            include_policy: true,
            include_checkpoints: true,
            include_final: true,
        },
    )
    .map_err(|error| error.to_string())?;
    Ok(render_trace_summary(&entries))
}

#[must_use]
pub fn render_trace_summary(entries: &[agent_core::TraceEntry]) -> String {
    entries
        .iter()
        .map(|entry| format!("{} {} {}\n", entry.seq, entry.kind, entry.summary))
        .collect()
}

#[must_use]
pub fn render_local_coding_summary(outcome: &agent_core::LocalCodingOutcome) -> String {
    let mut output = format!("{}\n", outcome.final_answer);
    output.push_str("Changed files:\n");
    if outcome.changed_files.is_empty() {
        output.push_str("- none\n");
    } else {
        for file in &outcome.changed_files {
            output.push_str(&format!("- {file}\n"));
        }
    }
    output.push_str("Command results:\n");
    if outcome.command_results.is_empty() {
        output.push_str("- none\n");
    } else {
        for result in &outcome.command_results {
            output.push_str(&format!("- {result}\n"));
        }
    }
    output.push_str(&format!("Audit events: {}\n", outcome.audit_event_count));
    output
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SlashCommand {
    Help,
    Status,
    Exit,
}

impl SlashCommand {
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim() {
            "/help" => Some(Self::Help),
            "/status" => Some(Self::Status),
            "/exit" => Some(Self::Exit),
            _ => None,
        }
    }
}

#[must_use]
pub fn handle_slash_command(input: &str) -> CliOutput {
    match SlashCommand::parse(input) {
        Some(SlashCommand::Help) => CliOutput::keep_running("Commands: /help, /status, /exit\n"),
        Some(SlashCommand::Status) => CliOutput::keep_running("Runtime: ready\n"),
        Some(SlashCommand::Exit) => CliOutput::exit("Bye\n"),
        None => CliOutput::keep_running(format!("Unknown command: {}\n", input.trim())),
    }
}
