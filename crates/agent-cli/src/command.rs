use thiserror::Error;

use crate::output::CliOutput;
use agent_model::{ModelConfigError, ModelEndpointConfig, ModelProviderConfig, ModelProviderKind};
use agent_storage::EventStore;
use agent_ui_bridge::{UiJsonFrame, encode_json_frame, normalize_runtime_event};

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
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => output_mode = OutputMode::Json,
            "--provider" => provider = Some(take_flag_value(&mut args, &arg)?),
            "--model" => model = Some(take_flag_value(&mut args, &arg)?),
            _ => command_args.push(arg),
        }
    }

    Ok(CliInvocation {
        command: parse_cli_command(command_args)?,
        output_mode,
        model_selection: ModelSelection::new(
            provider.unwrap_or_else(|| DEFAULT_MODEL_PROVIDER.to_string()),
            model.unwrap_or_else(|| DEFAULT_MODEL_NAME.to_string()),
        )?,
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
            CliOutput::exit(format!("agent-cli batch prompt: {prompt}\n"))
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
    }
}

#[must_use]
pub fn render_cli_invocation(invocation: CliInvocation) -> CliOutput {
    match invocation.output_mode {
        OutputMode::Text => render_cli_command(invocation.command),
        OutputMode::Json => render_json_invocation(invocation.command, invocation.model_selection),
    }
}

fn render_json_invocation(command: CliCommand, model_selection: ModelSelection) -> CliOutput {
    let mut stdout = render_model_selection_frame(&model_selection);

    match command {
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
        CliCommand::BatchPrompt { .. } => {
            stdout.push_str(&json_frame_line(UiJsonFrame::Done { event_count: 0 }));
            CliOutput::exit(stdout)
        }
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
