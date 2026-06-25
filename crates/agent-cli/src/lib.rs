#![forbid(unsafe_code)]
//! CLI command parsing and output contracts for agent.

pub mod command;
pub mod output;

pub use command::{
    CliCommand, CliError, SlashCommand, handle_slash_command, parse_cli_command, render_cli_command,
};
pub use output::CliOutput;
