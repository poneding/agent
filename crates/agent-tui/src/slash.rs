use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SlashCommand {
    Help,
    Status,
    Trace,
    Tasks,
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
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return Err(SlashCommandError::MissingSlash);
        }

        match trimmed.split_whitespace().next().unwrap_or(trimmed) {
            "/help" => Ok(Self::Help),
            "/status" => Ok(Self::Status),
            "/trace" => Ok(Self::Trace),
            "/tasks" => Ok(Self::Tasks),
            "/settings" => Ok(Self::Settings),
            "/clear" => Ok(Self::Clear),
            "/quit" | "/exit" => Ok(Self::Quit),
            command => Err(SlashCommandError::UnknownCommand {
                command: command.to_string(),
            }),
        }
    }
}
