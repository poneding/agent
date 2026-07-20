use agent_types::{
    ApprovalRequestId, CheckpointId, ClientKind, ConversationId, CorrelationId, SchemaVersion,
    TurnId, WorkspaceId,
};
use agent_ui_bridge::{
    UiApprovalDecision, UiBridgeError, UiClientInfo, UiCommand, UiCommandEnvelope,
    UiCommandTestSnapshot, validate_ui_command,
};

use tui_textarea::{CursorMove, TextArea};

use crate::{SlashCommand, SlashCommandError, SlashSuggestionState};

/// Dim placeholder shown in an empty composer.
pub const COMPOSER_PLACEHOLDER: &str = "type a message — / for commands · Enter to send";

/// Build a fresh, consistently-configured text area. The widget renders its own
/// cursor as a styled cell by default; we disable that (the render layer sets the
/// real terminal cursor from `TextArea::cursor()` instead, so there is exactly one
/// caret and it survives wrapped lines) and install the dim placeholder.
fn configured_textarea(initial: &str) -> TextArea<'static> {
    let mut textarea = if initial.is_empty() {
        TextArea::default()
    } else {
        TextArea::new(initial.split('\n').map(str::to_string).collect())
    };
    textarea.set_placeholder_text(COMPOSER_PLACEHOLDER);
    // Hide the widget's own block cursor; the render layer owns the terminal cursor.
    textarea.set_cursor_style(ratatui::style::Style::default());
    textarea.set_cursor_line_style(ratatui::style::Style::default());
    textarea
}

/// The interactive input line(s). Backed by `tui_textarea::TextArea`, which owns
/// multi-line editing, mid-line cursor motion, and word/line operations. The
/// history ring and slash-suggestion layer live on top and read the current text
/// via `text()`.
#[derive(Clone, Debug)]
pub struct ComposerState {
    /// The text editor widget: the source of truth for the composer's content and
    /// caret position.
    pub textarea: TextArea<'static>,
    /// Vestigial multiline flag retained for compatibility; newline insertion is
    /// now driven directly by Shift+Enter routing, not by a mode toggle.
    pub multiline: bool,
    pub history: Vec<String>,
    pub slash_suggestions: SlashSuggestionState,
    history_cursor: Option<usize>,
    /// In-progress draft preserved in a virtual slot while navigating history, so
    /// stepping past the newest entry restores what the user had typed.
    draft: Option<String>,
}

impl Default for ComposerState {
    fn default() -> Self {
        Self {
            textarea: configured_textarea(""),
            multiline: false,
            history: Vec::new(),
            slash_suggestions: SlashSuggestionState::default(),
            history_cursor: None,
            draft: None,
        }
    }
}

// `TextArea` implements neither `PartialEq` nor `Eq`, so we compare the observable
// state: the current text, the history ring, the suggestion layer, and the nav
// cursors. This keeps `assert_eq!` on `ComposerState`/`TuiApp` working while the
// editor internals (undo stack, styling) stay out of equality.
impl PartialEq for ComposerState {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text()
            && self.multiline == other.multiline
            && self.history == other.history
            && self.slash_suggestions == other.slash_suggestions
            && self.history_cursor == other.history_cursor
            && self.draft == other.draft
    }
}

impl Eq for ComposerState {}

impl ComposerState {
    /// The current composer contents, with multi-line input joined by `\n`.
    #[must_use]
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Whether the composer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.textarea.is_empty()
    }

    /// The caret position as `(row, column)` in character cells within the logical
    /// text (row = logical line index, column = char offset in that line).
    #[must_use]
    pub fn cursor(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    /// The number of logical lines currently in the composer (>= 1).
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.textarea.lines().len().max(1)
    }

    /// The composer's logical lines, for rendering. The render layer draws these
    /// directly (left-to-right) rather than handing the `TextArea` widget to
    /// ratatui, so its internal viewport can never reorder freshly typed text.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.textarea.lines().to_vec()
    }

    /// Replace the composer contents and move the caret to the end. Used by history
    /// navigation and slash-completion.
    fn set_text(&mut self, value: &str) {
        self.textarea = configured_textarea(value);
        self.textarea.move_cursor(CursorMove::Bottom);
        self.textarea.move_cursor(CursorMove::End);
    }

    pub fn reduce(&mut self, action: ComposerAction) -> TuiEffect {
        match action {
            ComposerAction::InsertText(text) => {
                // Bracketed paste arrives as one atomic event. Insert it verbatim
                // (including any embedded newlines) WITHOUT submitting — a pasted
                // multi-line string must never trigger a submit.
                self.history_cursor = None;
                self.textarea.insert_str(&text);
                self.refresh_slash_suggestions();
                TuiEffect::None
            }
            ComposerAction::InsertChar(character) => {
                self.history_cursor = None;
                self.textarea.insert_char(character);
                self.refresh_slash_suggestions();
                TuiEffect::None
            }
            ComposerAction::Backspace => {
                self.history_cursor = None;
                self.textarea.delete_char();
                self.refresh_slash_suggestions();
                TuiEffect::None
            }
            ComposerAction::InsertNewline => {
                // Shift+Enter (and the multiline toggle) inserts a literal newline
                // into the buffer; it never submits.
                self.history_cursor = None;
                self.multiline = true;
                self.textarea.insert_newline();
                self.refresh_slash_suggestions();
                TuiEffect::None
            }
            ComposerAction::ToggleMultiline => {
                self.multiline = !self.multiline;
                TuiEffect::None
            }
            ComposerAction::MoveCursor(motion) => {
                self.textarea.move_cursor(motion.into());
                TuiEffect::None
            }
            ComposerAction::Submit => self.submit(),
            ComposerAction::HistoryPrevious => self.history_previous(),
            ComposerAction::HistoryNext => self.history_next(),
            ComposerAction::Clear => {
                self.set_text("");
                self.history_cursor = None;
                self.refresh_slash_suggestions();
                TuiEffect::None
            }
            ComposerAction::Escape => {
                self.history_cursor = None;
                self.slash_suggestions = SlashSuggestionState::closed();
                TuiEffect::ComposerBlurred
            }
            ComposerAction::SlashSuggestionNext => {
                self.slash_suggestions.select_next();
                TuiEffect::None
            }
            ComposerAction::SlashSuggestionPrevious => {
                self.slash_suggestions.select_previous();
                TuiEffect::None
            }
            ComposerAction::SlashSuggestionComplete => self.complete_slash_suggestion(),
            ComposerAction::SlashSuggestionCompleteAndRun => {
                self.complete_and_run_slash_suggestion()
            }
        }
    }

    fn submit(&mut self) -> TuiEffect {
        let text = self.text();
        let prompt = text.trim();
        if prompt.is_empty() {
            // Submitting an empty prompt is a no-op; there is nothing useful to
            // surface to the user, so we silently ignore it rather than emitting
            // a diagnostic.
            return TuiEffect::None;
        }

        if prompt.starts_with('/') {
            return match SlashCommand::parse(prompt) {
                Ok(command) => {
                    self.set_text("");
                    self.history_cursor = None;
                    self.draft = None;
                    self.slash_suggestions = SlashSuggestionState::closed();
                    TuiEffect::from_slash_command(command)
                }
                Err(error) => TuiEffect::from_slash_error(error),
            };
        }

        let prompt = prompt.to_string();
        self.history.push(prompt.clone());
        self.set_text("");
        self.history_cursor = None;
        self.draft = None;
        self.slash_suggestions = SlashSuggestionState::closed();
        TuiEffect::SubmitPrompt(prompt)
    }

    fn history_previous(&mut self) -> TuiEffect {
        if self.history.is_empty() {
            return TuiEffect::None;
        }

        // Stepping into history for the first time: stash the in-progress draft in
        // the virtual slot so we can restore it when the user steps back past the
        // newest entry.
        if self.history_cursor.is_none() {
            self.draft = Some(self.text());
        }

        let next_index = match self.history_cursor {
            Some(0) => 0,
            Some(index) => index.saturating_sub(1),
            None => self.history.len() - 1,
        };

        self.history_cursor = Some(next_index);
        self.set_text(&self.history[next_index].clone());
        self.refresh_slash_suggestions();
        TuiEffect::None
    }

    fn history_next(&mut self) -> TuiEffect {
        let Some(index) = self.history_cursor else {
            return TuiEffect::None;
        };

        if index + 1 >= self.history.len() {
            // Stepped past the newest entry: restore the preserved draft.
            self.history_cursor = None;
            let draft = self.draft.take().unwrap_or_default();
            self.set_text(&draft);
        } else {
            let next_index = index + 1;
            self.history_cursor = Some(next_index);
            self.set_text(&self.history[next_index].clone());
        }

        self.refresh_slash_suggestions();

        TuiEffect::None
    }

    fn complete_slash_suggestion(&mut self) -> TuiEffect {
        let Some(command) = self
            .slash_suggestions
            .selected_command()
            .map(|cmd| cmd.to_string())
        else {
            return TuiEffect::None;
        };
        self.set_text(&format!("/{command}"));
        self.refresh_slash_suggestions();
        TuiEffect::None
    }

    /// Complete the selected slash suggestion and immediately run it, in one step.
    /// This is the Enter behavior on the slash menu (Tab = complete only). If there
    /// is no selection, falls back to a normal submit.
    fn complete_and_run_slash_suggestion(&mut self) -> TuiEffect {
        if self.slash_suggestions.selected_command().is_some() {
            self.complete_slash_suggestion();
        }
        self.submit()
    }

    fn refresh_slash_suggestions(&mut self) {
        self.slash_suggestions =
            SlashSuggestionState::from_input(&self.text(), self.slash_suggestions.selected_index());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComposerAction {
    InsertText(String),
    InsertChar(char),
    Backspace,
    InsertNewline,
    ToggleMultiline,
    Submit,
    HistoryPrevious,
    HistoryNext,
    Clear,
    Escape,
    /// Move the caret within the buffer (arrows / word / line motions) for
    /// mid-line editing without changing the text.
    MoveCursor(ComposerCursorMove),
    SlashSuggestionNext,
    SlashSuggestionPrevious,
    SlashSuggestionComplete,
    /// Complete the selected slash suggestion and run it in one step (Enter on the
    /// slash menu). `SlashSuggestionComplete` (Tab) only fills the composer.
    SlashSuggestionCompleteAndRun,
}

/// Caret motions the composer supports, mapped onto `tui_textarea::CursorMove`.
/// Kept as our own enum so `ComposerAction` stays `Eq`/`PartialEq` (the upstream
/// `CursorMove` is not) and the action vocabulary is independent of the widget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComposerCursorMove {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    WordForward,
    WordBack,
}

impl From<ComposerCursorMove> for CursorMove {
    fn from(value: ComposerCursorMove) -> Self {
        match value {
            ComposerCursorMove::Left => CursorMove::Back,
            ComposerCursorMove::Right => CursorMove::Forward,
            ComposerCursorMove::Up => CursorMove::Up,
            ComposerCursorMove::Down => CursorMove::Down,
            ComposerCursorMove::LineStart => CursorMove::Head,
            ComposerCursorMove::LineEnd => CursorMove::End,
            ComposerCursorMove::WordForward => CursorMove::WordForward,
            ComposerCursorMove::WordBack => CursorMove::WordBack,
        }
    }
}

/// Severity of a `LocalDiagnostic` — the client-side note/warning/error taxonomy.
/// This decides the *destination* and *color*: `Note` and `Warning` are transient,
/// neutral/dim toasts that auto-expire (no-op guidance, successes); only `Error` is
/// a real error that belongs in the transcript rendered red. A stray no-op must
/// never train the user to ignore red.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum LocalDiagnosticSeverity {
    /// Neutral guidance or a success confirmation (clipboard copy, "nothing to
    /// cancel"). Shown as a dim, auto-expiring toast. Never red.
    #[default]
    Note,
    /// A non-fatal caution. Shown as a toast (yellow accent), auto-expiring.
    Warning,
    /// A genuine error. Rendered in the transcript, red.
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TuiEffect {
    None,
    SubmitPrompt(String),
    SubmitRecoveryContinuation {
        prompt: String,
        summary: String,
    },
    RunSlashCommand(SlashCommand),
    /// A locally-generated message for the user. The `severity` decides where it
    /// goes and how it looks: `Note`/`Warning` become a transient, neutral/dim
    /// toast (no-op guidance and successes — never a red chip), while `Error`
    /// records a real error in the transcript. Prefer the `note()` / `warning()` /
    /// `error_diagnostic()` constructors over building this directly.
    LocalDiagnostic {
        message: String,
        severity: LocalDiagnosticSeverity,
    },
    ComposerBlurred,
    CopyToClipboard {
        text: String,
    },
    PersistTerminalTheme {
        theme: String,
    },
    ResolveApproval {
        request_id: ApprovalRequestId,
        decision: UiApprovalDecision,
        reason: String,
        lease_duration_seconds: Option<u64>,
    },
    ApprovalAborted(String),
    CommandRetryPlan {
        command_run_id: String,
        reason: String,
        dry_run: bool,
        evidence: Option<Box<UiCommandTestSnapshot>>,
    },
    ConversationCompact {
        max_summary_tokens: u32,
    },
    RecoveryStatus {
        turn_id: Option<TurnId>,
    },
    RecoveryResumeDryRun {
        turn_id: Option<TurnId>,
        checkpoint_id: Option<CheckpointId>,
    },
    RecoveryResume {
        turn_id: Option<TurnId>,
        checkpoint_id: Option<CheckpointId>,
    },
    SwitchSession {
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
    },
    /// Switch the active model provider (and its default model) for subsequent
    /// turns. Handled by the command runtime, which updates the shared config
    /// override and echoes a `ModelSelection` frame back to refresh the UI.
    SwitchModel {
        provider_id: String,
        model_id: String,
    },
    /// Start a fresh conversation in the current workspace. Handled by the
    /// command runtime, which points the shared identity at a new conversation
    /// and echoes a refreshed session snapshot back to the UI.
    NewSession,
}

impl TuiEffect {
    #[must_use]
    pub const fn from_slash_command(command: SlashCommand) -> Self {
        Self::RunSlashCommand(command)
    }

    #[must_use]
    pub fn from_slash_error(error: SlashCommandError) -> Self {
        // A malformed slash command is a real error, so it renders red.
        Self::error_diagnostic(error.to_string())
    }

    /// A neutral, dim no-op note or success confirmation (clipboard copy,
    /// "nothing to cancel"). Routed to a transient toast, never a red chip.
    #[must_use]
    pub fn note(message: impl Into<String>) -> Self {
        Self::LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Note,
            message: message.into(),
        }
    }

    /// A warning: something the user should notice but not a failure.
    #[must_use]
    pub fn warning(message: impl Into<String>) -> Self {
        Self::LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Warning,
            message: message.into(),
        }
    }

    /// A real error. This is the only severity that renders red.
    #[must_use]
    pub fn error_diagnostic(message: impl Into<String>) -> Self {
        Self::LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Error,
            message: message.into(),
        }
    }

    pub fn to_ui_command(
        &self,
        context: &TuiBridgeContext,
    ) -> Result<Option<UiCommandEnvelope>, UiBridgeError> {
        let command = match self {
            Self::SubmitPrompt(prompt) | Self::SubmitRecoveryContinuation { prompt, .. } => {
                UiCommand::StartTurn {
                    workspace_id: context.workspace_id,
                    conversation_id: context.conversation_id,
                    prompt: prompt.clone(),
                }
            }
            Self::ResolveApproval {
                request_id,
                decision,
                reason,
                lease_duration_seconds,
            } => UiCommand::ResolveApproval {
                request_id: *request_id,
                decision: *decision,
                reason: reason.clone(),
                lease_duration_seconds: *lease_duration_seconds,
            },
            Self::SwitchSession {
                workspace_id,
                conversation_id,
            } => UiCommand::SwitchSession {
                workspace_id: *workspace_id,
                conversation_id: *conversation_id,
            },
            Self::CommandRetryPlan {
                command_run_id,
                reason,
                dry_run,
                evidence: _,
            } => UiCommand::CommandRetryPlan {
                command_run_id: command_run_id.clone(),
                reason: reason.clone(),
                dry_run: *dry_run,
            },
            Self::ConversationCompact { max_summary_tokens } => UiCommand::ConversationCompact {
                conversation_id: context.conversation_id,
                max_summary_tokens: *max_summary_tokens,
            },
            Self::RecoveryStatus { turn_id } => UiCommand::RecoveryStatus {
                conversation_id: context.conversation_id,
                turn_id: *turn_id,
            },
            Self::RecoveryResumeDryRun {
                turn_id,
                checkpoint_id,
            } => UiCommand::RecoveryResumeDryRun {
                conversation_id: context.conversation_id,
                turn_id: *turn_id,
                checkpoint_id: *checkpoint_id,
            },
            Self::RecoveryResume {
                turn_id,
                checkpoint_id,
            } => UiCommand::RecoveryResume {
                conversation_id: context.conversation_id,
                turn_id: *turn_id,
                checkpoint_id: *checkpoint_id,
            },
            Self::CopyToClipboard { .. }
            | Self::PersistTerminalTheme { .. }
            | Self::ApprovalAborted(_)
            | Self::SwitchModel { .. }
            | Self::NewSession
            | Self::None
            | Self::RunSlashCommand(_)
            | Self::LocalDiagnostic { .. }
            | Self::ComposerBlurred => return Ok(None),
        };

        let envelope = UiCommandEnvelope {
            schema_version: SchemaVersion::current(),
            command_id: CorrelationId::new(),
            client: UiClientInfo {
                kind: context.client_kind.clone(),
                version: context.client_version.clone(),
            },
            command,
        };

        validate_ui_command(&envelope)?;
        Ok(Some(envelope))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiBridgeContext {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub client_kind: ClientKind,
    pub client_version: String,
}

impl TuiBridgeContext {
    pub fn new(
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
        client_kind: ClientKind,
        client_version: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let client_version = client_version.into();
        if client_version.trim().is_empty() {
            return Err(UiBridgeError::Validation(
                "tui bridge client version cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            workspace_id,
            conversation_id,
            client_kind,
            client_version,
        })
    }
}
