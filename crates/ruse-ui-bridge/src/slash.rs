use serde::{Deserialize, Serialize};
use thiserror::Error;

const PUBLIC_TEXT_LIMIT: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiSlashCommandCatalogSnapshot {
    pub commands: Vec<UiSlashCommandDescriptor>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiSlashCommandDescriptor {
    pub name: String,
    pub aliases: Vec<String>,
    pub category: UiSlashCommandCategory,
    pub summary: String,
    pub effect: UiSlashCommandEffect,
    pub available_in: Vec<UiSlashCommandSurface>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSlashCommandCategory {
    General,
    Status,
    Sessions,
    Navigation,
    Tools,
    Utility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSlashCommandEffect {
    OpenHelp,
    ShowStatus,
    ShowModel,
    ShowPermissions,
    OpenSessions,
    NewSession,
    ResumeSession,
    OpenTrace,
    OpenTasks,
    OpenPlan,
    OpenDiff,
    OpenTools,
    OpenMcp,
    OpenMemory,
    OpenTheme,
    ConfigureThinking,
    CompactConversation,
    RetryLastTurn,
    ClearTranscript,
    Quit,
    OpenSettings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSlashCommandSurface {
    PlainRepl,
    Tui,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum UiSlashCommandParseError {
    #[error("slash command must start with /")]
    MissingSlash,
    #[error("unknown slash command: {command}")]
    UnknownCommand { command: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiTerminalStatusSnapshot {
    pub provider: String,
    pub model: String,
    pub workspace: String,
    pub mode: String,
    pub connection: String,
    pub focused_panel: String,
    pub session_count: u32,
    pub diagnostic_count: u32,
}

impl UiTerminalStatusSnapshot {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        workspace: impl Into<String>,
        mode: impl Into<String>,
        connection: impl Into<String>,
        focused_panel: impl Into<String>,
        session_count: u32,
        diagnostic_count: u32,
    ) -> Self {
        Self {
            provider: bounded_public_text(&provider.into(), "unknown"),
            model: bounded_public_text(&model.into(), "unknown"),
            workspace: bounded_public_text(&workspace.into(), "unknown"),
            mode: bounded_public_text(&mode.into(), "unknown"),
            connection: bounded_public_text(&connection.into(), "unknown"),
            focused_panel: bounded_public_text(&focused_panel.into(), "none"),
            session_count,
            diagnostic_count,
        }
    }
}

#[must_use]
pub fn default_slash_command_catalog() -> UiSlashCommandCatalogSnapshot {
    let surfaces = vec![UiSlashCommandSurface::PlainRepl, UiSlashCommandSurface::Tui];
    let tui_only = vec![UiSlashCommandSurface::Tui];
    let mut commands = vec![
        descriptor(
            "help",
            Vec::new(),
            UiSlashCommandCategory::General,
            "Show available slash commands",
            UiSlashCommandEffect::OpenHelp,
            surfaces.clone(),
        ),
        descriptor(
            "status",
            Vec::new(),
            UiSlashCommandCategory::Status,
            "Show terminal runtime status",
            UiSlashCommandEffect::ShowStatus,
            surfaces.clone(),
        ),
        descriptor(
            "model",
            Vec::new(),
            UiSlashCommandCategory::Status,
            "Switch the active model provider",
            UiSlashCommandEffect::ShowModel,
            surfaces.clone(),
        ),
        descriptor(
            "permissions",
            vec!["permission"],
            UiSlashCommandCategory::Status,
            "Show permission mode and approval state",
            UiSlashCommandEffect::ShowPermissions,
            surfaces.clone(),
        ),
        descriptor(
            "sessions",
            Vec::new(),
            UiSlashCommandCategory::Sessions,
            "Show terminal sessions",
            UiSlashCommandEffect::OpenSessions,
            surfaces.clone(),
        ),
        descriptor(
            "new",
            Vec::new(),
            UiSlashCommandCategory::Sessions,
            "Start a new terminal session in this workspace",
            UiSlashCommandEffect::NewSession,
            surfaces.clone(),
        ),
        descriptor(
            "resume",
            Vec::new(),
            UiSlashCommandCategory::Sessions,
            "Resume or switch to a recent terminal session",
            UiSlashCommandEffect::ResumeSession,
            surfaces.clone(),
        ),
        descriptor(
            "trace",
            Vec::new(),
            UiSlashCommandCategory::Navigation,
            "Show runtime trace",
            UiSlashCommandEffect::OpenTrace,
            surfaces.clone(),
        ),
        descriptor(
            "tasks",
            Vec::new(),
            UiSlashCommandCategory::Navigation,
            "Show task status",
            UiSlashCommandEffect::OpenTasks,
            surfaces.clone(),
        ),
        descriptor(
            "plan",
            Vec::new(),
            UiSlashCommandCategory::Navigation,
            "Show plan and task progress",
            UiSlashCommandEffect::OpenPlan,
            surfaces.clone(),
        ),
        descriptor(
            "diff",
            Vec::new(),
            UiSlashCommandCategory::Navigation,
            "Show latest edit diff",
            UiSlashCommandEffect::OpenDiff,
            surfaces.clone(),
        ),
        descriptor(
            "tools",
            Vec::new(),
            UiSlashCommandCategory::Tools,
            "Show tool activity",
            UiSlashCommandEffect::OpenTools,
            surfaces.clone(),
        ),
        descriptor(
            "mcp",
            Vec::new(),
            UiSlashCommandCategory::Tools,
            "Show MCP server and tool activity",
            UiSlashCommandEffect::OpenMcp,
            surfaces.clone(),
        ),
        descriptor(
            "memory",
            Vec::new(),
            UiSlashCommandCategory::Status,
            "Show memory state",
            UiSlashCommandEffect::OpenMemory,
            surfaces.clone(),
        ),
        descriptor(
            "theme",
            Vec::new(),
            UiSlashCommandCategory::Utility,
            "Show terminal theme status",
            UiSlashCommandEffect::OpenTheme,
            surfaces.clone(),
        ),
        descriptor(
            "thinking",
            Vec::new(),
            UiSlashCommandCategory::Utility,
            "Manage history: /thinking show-all|hide-all",
            UiSlashCommandEffect::ConfigureThinking,
            surfaces.clone(),
        ),
        descriptor(
            "compact",
            Vec::new(),
            UiSlashCommandCategory::Utility,
            "Compact the current conversation",
            UiSlashCommandEffect::CompactConversation,
            surfaces.clone(),
        ),
        descriptor(
            "retry",
            Vec::new(),
            UiSlashCommandCategory::Utility,
            "Retry the last failed turn",
            UiSlashCommandEffect::RetryLastTurn,
            surfaces.clone(),
        ),
        descriptor(
            "clear",
            Vec::new(),
            UiSlashCommandCategory::Utility,
            "Clear local transcript state",
            UiSlashCommandEffect::ClearTranscript,
            surfaces.clone(),
        ),
        descriptor(
            "quit",
            vec!["exit", "bye"],
            UiSlashCommandCategory::Utility,
            "Exit the terminal session",
            UiSlashCommandEffect::Quit,
            surfaces,
        ),
        descriptor(
            "settings",
            Vec::new(),
            UiSlashCommandCategory::Utility,
            "Show settings status",
            UiSlashCommandEffect::OpenSettings,
            tui_only,
        ),
    ];
    commands.sort_by(|left, right| {
        category_order(left.category)
            .cmp(&category_order(right.category))
            .then_with(|| left.name.cmp(&right.name))
    });
    UiSlashCommandCatalogSnapshot { commands }
}

pub fn resolve_slash_command(
    input: &str,
) -> Result<UiSlashCommandDescriptor, UiSlashCommandParseError> {
    resolve_slash_command_in_catalog(&default_slash_command_catalog(), input).cloned()
}

pub fn resolve_slash_command_in_catalog<'a>(
    catalog: &'a UiSlashCommandCatalogSnapshot,
    input: &str,
) -> Result<&'a UiSlashCommandDescriptor, UiSlashCommandParseError> {
    let token = slash_token(input)?;
    catalog
        .commands
        .iter()
        .find(|command| command.name == token || command.aliases.iter().any(|alias| alias == token))
        .ok_or_else(|| UiSlashCommandParseError::UnknownCommand {
            command: format!("/{}", bounded_public_text(token, "unknown")),
        })
}

fn descriptor(
    name: impl Into<String>,
    aliases: Vec<&str>,
    category: UiSlashCommandCategory,
    summary: impl Into<String>,
    effect: UiSlashCommandEffect,
    available_in: Vec<UiSlashCommandSurface>,
) -> UiSlashCommandDescriptor {
    UiSlashCommandDescriptor {
        name: name.into(),
        aliases: aliases.into_iter().map(str::to_string).collect(),
        category,
        summary: bounded_public_text(&summary.into(), "no summary"),
        effect,
        available_in,
    }
}

fn slash_token(input: &str) -> Result<&str, UiSlashCommandParseError> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return Err(UiSlashCommandParseError::MissingSlash);
    }
    let token = trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .trim_start_matches('/');
    if token.is_empty() {
        return Err(UiSlashCommandParseError::MissingSlash);
    }
    Ok(token)
}

fn category_order(category: UiSlashCommandCategory) -> u8 {
    match category {
        UiSlashCommandCategory::General => 0,
        UiSlashCommandCategory::Status => 1,
        UiSlashCommandCategory::Sessions => 2,
        UiSlashCommandCategory::Navigation => 3,
        UiSlashCommandCategory::Tools => 4,
        UiSlashCommandCategory::Utility => 5,
    }
}

fn bounded_public_text(value: &str, fallback: &str) -> String {
    let redacted = redact_secret_like_text(value);
    let trimmed = redacted.trim();
    let value = if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    };
    if value.chars().count() > PUBLIC_TEXT_LIMIT {
        value.chars().take(PUBLIC_TEXT_LIMIT).collect()
    } else {
        value.to_string()
    }
}

fn redact_secret_like_text(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            if is_secret_like(word) {
                "[redacted]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}
