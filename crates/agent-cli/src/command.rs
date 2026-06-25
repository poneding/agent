use thiserror::Error;

use crate::output::CliOutput;
use agent_storage::EventStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Version,
    RuntimeBanner,
    BatchPrompt { prompt: String },
    TtyShell,
    LocalSummary,
    TraceSummary,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CliError {
    #[error("unknown argument: {0}")]
    UnknownArgument(String),
    #[error("missing value for argument: {0}")]
    MissingValue(String),
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
