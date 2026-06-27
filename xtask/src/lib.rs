#![forbid(unsafe_code)]
//! Shared parsing and command planning for the workspace xtask entrypoint.

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Task {
    Help,
    Check,
    Fmt,
    FmtCheck,
    Clippy,
    Test,
    CliPrompt { prompt: String },
    CliJson { prompt: String },
    DesktopTest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl CargoCommand {
    fn new(args: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            program: "cargo".to_string(),
            args: args.into_iter().map(str::to_string).collect(),
        }
    }

    fn with_prompt(args: &[&str], prompt: &str) -> Self {
        let mut command = Self {
            program: "cargo".to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        };
        command.args.push(prompt.to_string());
        command
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XtaskError {
    UnknownCommand(String),
    MissingPrompt(String),
}

impl fmt::Display for XtaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(command) => write!(formatter, "unknown xtask command: {command}"),
            Self::MissingPrompt(command) => write!(formatter, "{command} requires a prompt"),
        }
    }
}

impl std::error::Error for XtaskError {}

pub fn parse_args<I, S>(args: I) -> Result<Task, XtaskError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let Some(command) = args.next() else {
        return Ok(Task::Help);
    };

    match command.as_str() {
        "help" | "-h" | "--help" => Ok(Task::Help),
        "check" => Ok(Task::Check),
        "fmt" => Ok(Task::Fmt),
        "fmt-check" => Ok(Task::FmtCheck),
        "clippy" => Ok(Task::Clippy),
        "test" => Ok(Task::Test),
        "desktop-test" => Ok(Task::DesktopTest),
        "cli-prompt" => Ok(Task::CliPrompt {
            prompt: parse_prompt(&command, args.collect())?,
        }),
        "cli-json" => Ok(Task::CliJson {
            prompt: parse_prompt(&command, args.collect())?,
        }),
        _ => Err(XtaskError::UnknownCommand(command)),
    }
}

#[must_use]
pub fn command_plan_for(task: &Task) -> Vec<CargoCommand> {
    match task {
        Task::Help => Vec::new(),
        Task::Check => vec![fmt_check_command(), clippy_command(), test_command()],
        Task::Fmt => vec![CargoCommand::new(["fmt", "--all"])],
        Task::FmtCheck => vec![fmt_check_command()],
        Task::Clippy => vec![clippy_command()],
        Task::Test => vec![test_command()],
        Task::CliPrompt { prompt } => vec![CargoCommand::with_prompt(
            &["run", "-p", "agent-cli", "--", "--prompt"],
            prompt,
        )],
        Task::CliJson { prompt } => vec![CargoCommand::with_prompt(
            &["run", "-p", "agent-cli", "--", "--json", "--prompt"],
            prompt,
        )],
        Task::DesktopTest => vec![CargoCommand::new(["test", "-p", "agent-desktop"])],
    }
}

#[must_use]
pub fn help_text() -> &'static str {
    r#"cargo xtask <command>

Commands:
  check                 Run fmt-check, clippy, and test
  fmt                   Run cargo fmt --all
  fmt-check             Run cargo fmt --all -- --check
  clippy                Run cargo clippy --workspace --all-targets -- -D warnings
  test                  Run cargo test --workspace
  cli-prompt -- <text>  Run agent-cli with --prompt <text>
  cli-json -- <text>    Run agent-cli with --json --prompt <text>
  desktop-test          Run cargo test -p agent-desktop
  help                  Print this help
"#
}

fn parse_prompt(command: &str, words: Vec<String>) -> Result<String, XtaskError> {
    let words = match words.split_first() {
        Some((separator, rest)) if separator == "--" => rest,
        _ => words.as_slice(),
    };

    if words.is_empty() {
        return Err(XtaskError::MissingPrompt(command.to_string()));
    }

    Ok(words.join(" "))
}

fn fmt_check_command() -> CargoCommand {
    CargoCommand::new(["fmt", "--all", "--", "--check"])
}

fn clippy_command() -> CargoCommand {
    CargoCommand::new([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])
}

fn test_command() -> CargoCommand {
    CargoCommand::new(["test", "--workspace"])
}
