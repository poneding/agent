use ruse_ui_bridge::{UiSlashCommandEffect, UiSlashCommandParseError, resolve_slash_command};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ThinkingVisibilityCommand {
    Show,
    Hide,
    Toggle,
    ShowAll,
    HideAll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashSuggestionItem {
    pub name: String,
    pub description: String,
    pub summary: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SlashSuggestionState {
    pub query: String,
    pub prefix: String,
    pub items: Vec<SlashSuggestionItem>,
    /// Render/selection surface expects this name.
    pub matches: Vec<SlashSuggestionItem>,
    pub selected: usize,
}

impl SlashSuggestionState {
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn visible(&self) -> bool {
        !self.query.is_empty() && self.query.starts_with('/')
    }

    #[must_use]
    pub fn from_input(text: &str, selected_index: usize) -> Self {
        let query = text.trim().to_string();
        if !query.starts_with('/') {
            return Self::default();
        }
        let prefix = query.trim_start_matches('/').to_string();
        let q = prefix.to_ascii_lowercase();
        let catalog = [
            ("help", "Show help"),
            ("status", "Show status"),
            ("model", "Pick model"),
            ("diff", "Show diff"),
            ("plan", "Show plan"),
            ("sessions", "Sessions"),
            ("tools", "List tools"),
            ("mcp", "MCP status"),
            ("memory", "Memory"),
            ("theme", "Theme"),
            ("clear", "Clear transcript"),
            ("compact", "Compact conversation"),
            ("copy", "Copy latest output"),
            ("retry", "Retry last command"),
            ("quit", "Quit"),
        ];
        let mut items: Vec<SlashSuggestionItem> = catalog
            .into_iter()
            .filter(|(name, _)| name.starts_with(&q) || q.is_empty())
            .map(|(name, description)| SlashSuggestionItem {
                name: name.to_string(),
                description: description.to_string(),
                summary: description.to_string(),
                selected: false,
            })
            .collect();
        let selected = if items.is_empty() {
            0
        } else {
            selected_index.min(items.len() - 1)
        };
        if let Some(item) = items.get_mut(selected) {
            item.selected = true;
        }
        let matches = items.clone();
        Self {
            query,
            prefix,
            items,
            matches,
            selected,
        }
    }

    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn select_next(&mut self) {
        if self.matches.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1) % self.matches.len();
        self.resync_selected_flags();
    }

    pub fn select_previous(&mut self) {
        if self.matches.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = if self.selected == 0 {
            self.matches.len() - 1
        } else {
            self.selected - 1
        };
        self.resync_selected_flags();
    }

    pub fn select_prev(&mut self) {
        self.select_previous();
    }

    #[must_use]
    pub fn selected_item(&self) -> Option<&SlashSuggestionItem> {
        self.matches.get(self.selected)
    }

    #[must_use]
    pub fn selected_command(&self) -> Option<SlashCommand> {
        let name = self.selected_item()?.name.trim_start_matches('/');
        SlashCommand::parse(&format!("/{name}")).ok()
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        *self = Self::from_input(&query.into(), 0);
    }

    fn resync_selected_flags(&mut self) {
        for (idx, item) in self.matches.iter_mut().enumerate() {
            item.selected = idx == self.selected;
        }
        self.items = self.matches.clone();
    }
}

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
    Model,
    Diff,
    Plan,
    Permissions,
    Resume,
    Compact,
    Thinking(ThinkingVisibilityCommand),
    Tools,
    Mcp,
    Memory,
    Theme,
    Copy,
    Retry,
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
        use UiSlashCommandEffect::*;
        match effect {
            OpenHelp => Some(Self::Help),
            ShowStatus => Some(Self::Status),
            OpenTrace => Some(Self::Trace),
            OpenTasks => Some(Self::Tasks),
            OpenSessions => Some(Self::Sessions),
            OpenSettings => Some(Self::Settings),
            ClearTranscript => Some(Self::Clear),
            Quit => Some(Self::Quit),
            ShowModel => Some(Self::Model),
            OpenDiff => Some(Self::Diff),
            OpenPlan => Some(Self::Plan),
            ShowPermissions => Some(Self::Permissions),
            ResumeSession => Some(Self::Resume),
            CompactConversation => Some(Self::Compact),
            ConfigureThinking => Some(Self::Thinking(ThinkingVisibilityCommand::Toggle)),
            OpenTools => Some(Self::Tools),
            OpenMcp => Some(Self::Mcp),
            OpenMemory => Some(Self::Memory),
            OpenTheme => Some(Self::Theme),
            CopyLatestOutput => Some(Self::Copy),
            RetryLastTurn => Some(Self::Retry),
        }
    }
}

impl From<UiSlashCommandParseError> for SlashCommandError {
    fn from(value: UiSlashCommandParseError) -> Self {
        match value {
            UiSlashCommandParseError::MissingSlash => Self::MissingSlash,
            UiSlashCommandParseError::UnknownCommand { command } => Self::UnknownCommand { command },
        }
    }
}

impl std::fmt::Display for SlashCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Help => "help",
            Self::Status => "status",
            Self::Trace => "trace",
            Self::Tasks => "tasks",
            Self::Sessions => "sessions",
            Self::Settings => "settings",
            Self::Clear => "clear",
            Self::Quit => "quit",
            Self::Model => "model",
            Self::Diff => "diff",
            Self::Plan => "plan",
            Self::Permissions => "permissions",
            Self::Resume => "resume",
            Self::Compact => "compact",
            Self::Thinking(_) => "thinking",
            Self::Tools => "tools",
            Self::Mcp => "mcp",
            Self::Memory => "memory",
            Self::Theme => "theme",
            Self::Copy => "copy",
            Self::Retry => "retry",
        };
        f.write_str(name)
    }
}
