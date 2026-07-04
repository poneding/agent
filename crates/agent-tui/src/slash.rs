use agent_ui_bridge::{UiSlashCommandEffect, UiSlashCommandParseError, resolve_slash_command};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SlashCommand {
    Help,
    Status,
    Trace,
    Tasks,
    Sessions,
    Settings,
    Clear,
    Quit,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SlashCommandError {
    #[error("slash command must start with /")]
    MissingSlash,
    #[error("unknown slash command: {command}")]
    UnknownCommand { command: String },
}

impl SlashCommand {
    pub fn parse(input: &str) -> Result<Self, SlashCommandError> {
        let descriptor = resolve_slash_command(input).map_err(SlashCommandError::from)?;
        Self::from_effect(descriptor.effect).ok_or(SlashCommandError::UnknownCommand {
            command: format!("/{}", descriptor.name),
        })
    }

    fn from_effect(effect: UiSlashCommandEffect) -> Option<Self> {
        match effect {
            UiSlashCommandEffect::OpenHelp => Some(Self::Help),
            UiSlashCommandEffect::ShowStatus => Some(Self::Status),
            UiSlashCommandEffect::OpenTrace => Some(Self::Trace),
            UiSlashCommandEffect::OpenTasks => Some(Self::Tasks),
            UiSlashCommandEffect::OpenSessions => Some(Self::Sessions),
            UiSlashCommandEffect::OpenSettings => Some(Self::Settings),
            UiSlashCommandEffect::ClearTranscript => Some(Self::Clear),
            UiSlashCommandEffect::Quit => Some(Self::Quit),
        }
    }
}

impl From<UiSlashCommandParseError> for SlashCommandError {
    fn from(value: UiSlashCommandParseError) -> Self {
        match value {
            UiSlashCommandParseError::MissingSlash => Self::MissingSlash,
            UiSlashCommandParseError::UnknownCommand { command } => {
                Self::UnknownCommand { command }
            }
        }
    }
}
