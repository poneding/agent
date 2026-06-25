use thiserror::Error;

use crate::output::CliOutput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Version,
    RuntimeBanner,
    BatchPrompt { prompt: String },
    TtyShell,
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
    }
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
