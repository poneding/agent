use std::collections::BTreeSet;
use std::time::{Duration, Instant};

pub use ruse_types::ReasoningEffort;
use ruse_types::{ConversationId, RecoveryContinuation};
use ruse_ui_bridge::{UiApprovalRequest, UiCommandTestSnapshot, UiEventPayload, UiJsonFrame};

use crate::{
    ApprovalPanelAction, ApprovalPanelState, CommandPanelAction, CommandPanelState, ComposerAction,
    ComposerState, DiffPanelAction, DiffPanelState, ExtensionPanelState, LocalDiagnosticSeverity,
    ModelPickerAction, ModelPickerState, SelectionState, SessionPanelAction, SessionPanelState,
    SlashCommand, SlashSuggestionState, StatusPanelState, TaskPanelState, ThemePickerAction,
    ThemePickerState, ThinkingVisibilityCommand, TracePanelState, TranscriptState, TuiEffect,
    WaitingActivity,
};

const DEFAULT_COMPACTION_SUMMARY_TOKENS: u32 = 256;

/// Number of frames in the live activity spinner animation. Kept in sync with
/// the frame table in `render.rs` (`WAITING_SPINNER_FRAMES`).
pub const SPINNER_FRAME_COUNT: usize = 6;

/// Wall-clock milliseconds between spinner frame advances. ~12fps, matching the
/// cadence used by Claude Code / Codex. The event loop recomputes the frame
/// index from elapsed time so the animation is independent of poll frequency.
pub const SPINNER_FRAME_INTERVAL_MS: u128 = 80;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TuiScreen {
    Chat,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TuiSidePanel {
    #[default]
    Auto,
    Trace,
    Tasks,
    Status,
    Extensions,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TerminalTheme {
    #[default]
    System,
    Light,
    Dark,
    HighContrast,
}

impl TerminalTheme {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::HighContrast => "high-contrast",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::HighContrast,
            Self::HighContrast => Self::System,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextWindowState {
    pub used_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

impl ContextWindowState {
    /// Default width of the compact context progress bar in the status footer.
    pub const FOOTER_BAR_WIDTH: usize = 8;

    #[must_use]
    pub const fn pending(total_tokens: Option<u32>) -> Self {
        Self {
            used_tokens: None,
            total_tokens,
        }
    }

    /// `used/total` token counts for surfaces that want the raw numbers
    /// (welcome card, status panel). The footer prefers [`Self::render_bar`].
    #[must_use]
    pub fn render(&self) -> String {
        let used = self
            .used_tokens
            .map(format_token_count)
            .unwrap_or_else(|| "pending".to_string());
        let total = self
            .total_tokens
            .map(format_token_count)
            .unwrap_or_else(|| "unknown".to_string());
        format!("{used}/{total}")
    }

    /// Usage as a 0–100 percentage when both sides of the window are known.
    #[must_use]
    pub fn usage_percent(&self) -> Option<u8> {
        let (Some(used), Some(total)) = (self.used_tokens, self.total_tokens) else {
            return None;
        };
        if total == 0 {
            return None;
        }
        // Rounded percent, clamped so a slightly-over-budget estimate still
        // reads as 100 rather than wrapping.
        let pct = (u64::from(used) * 100 + u64::from(total) / 2) / u64::from(total);
        Some(pct.min(100) as u8)
    }

    /// Compact context meter for the status footer: a short `█`/`░` bar followed
    /// by the original `used/total` token counts (no `ctx` prefix). When usage
    /// is still unknown the bar is omitted and only the counts are shown.
    #[must_use]
    pub fn render_bar(&self) -> String {
        self.render_bar_with_width(Self::FOOTER_BAR_WIDTH)
    }

    #[must_use]
    pub fn render_bar_with_width(&self, width: usize) -> String {
        let counts = self.render();
        let width = width.max(4);
        let (Some(used), Some(total)) = (self.used_tokens, self.total_tokens) else {
            return counts;
        };
        if total == 0 {
            return counts;
        }
        let filled = ((u64::from(used) * width as u64) / u64::from(total)) as usize;
        let filled = filled.min(width);
        let mut bar = String::with_capacity(width);
        for i in 0..width {
            bar.push(if i < filled { '█' } else { '░' });
        }
        format!("{bar} {counts}")
    }
}

impl Default for ContextWindowState {
    fn default() -> Self {
        Self::pending(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusBarState {
    pub provider: String,
    pub model: String,
    pub workspace: String,
    pub mode: String,
    pub connection: String,
    pub permission_state: String,
    pub reasoning_effort: ReasoningEffort,
    pub context: ContextWindowState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRetryCandidateState {
    pub command_run_id: String,
    pub command: String,
    pub status: String,
    pub classification: String,
    pub evidence: UiCommandTestSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryContinuationState {
    pub summary: String,
    pub next_prompt: String,
    pub can_start_turn: bool,
}

impl RecoveryContinuationState {
    #[must_use]
    pub fn from_continuation(continuation: &RecoveryContinuation) -> Self {
        Self {
            summary: continuation.summary.clone(),
            next_prompt: continuation.next_prompt.clone(),
            can_start_turn: continuation.can_start_turn,
        }
    }
}

impl CommandRetryCandidateState {
    #[must_use]
    pub fn from_command_test_snapshot(snapshot: &UiCommandTestSnapshot) -> Option<Self> {
        let command_run_id = snapshot.command_run_id.trim();
        if command_run_id.is_empty() || !is_retry_candidate(snapshot) {
            return None;
        }

        Some(Self {
            command_run_id: command_run_id.to_string(),
            command: snapshot.command.clone(),
            status: snapshot.status.clone(),
            classification: snapshot.classification.clone(),
            evidence: snapshot.clone(),
        })
    }
}

impl StatusBarState {
    #[must_use]
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        workspace: impl Into<String>,
        mode: impl Into<String>,
        connection: impl Into<String>,
        permission_state: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            workspace: workspace.into(),
            mode: mode.into(),
            connection: connection.into(),
            permission_state: permission_state.into(),
            reasoning_effort: ReasoningEffort::default(),
            context: ContextWindowState::default(),
        }
    }

    #[must_use]
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = effort;
        self
    }

    #[must_use]
    pub fn with_context_total(mut self, total_tokens: Option<u32>) -> Self {
        self.context.total_tokens = total_tokens;
        self
    }
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self::new(
            "fake",
            "fake-scripted",
            ".",
            "tui",
            "disconnected",
            "default-deny",
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TuiAction {
    Tick,
    QuitRequested,
    CancelTurnRequested,
    Resize {
        width: u16,
        height: u16,
    },
    ClearTranscript,
    TogglePanelExpansion,
    /// Toggle only the current or most recent reasoning block. Other historical
    /// thinking items keep their independent visibility state.
    ToggleReasoning,
    /// Scroll the transcript up a few lines (mouse wheel).
    ScrollTranscriptUp,
    /// Scroll the transcript down a few lines (mouse wheel).
    ScrollTranscriptDown,
    /// Scroll the transcript up by roughly one viewport (PageUp).
    ScrollTranscriptPageUp,
    /// Scroll the transcript down by roughly one viewport (PageDown).
    ScrollTranscriptPageDown,
    ScrollTranscriptTop,
    ScrollTranscriptBottom,
    /// Expand/collapse the latest tool call/result output (Ctrl+O).
    ToggleToolOutput,
    /// Expand/collapse a specific reasoning item (mouse click).
    ToggleReasoningAt {
        index: usize,
    },
    /// Expand/collapse a specific tool item's output (mouse click).
    ToggleToolOutputAt {
        index: usize,
    },
    /// Pointer moved to terminal-absolute coordinates (hover tracking).
    PointerMove {
        column: u16,
        row: u16,
    },
    /// Left-click at terminal-absolute coordinates; hit-tested against the
    /// transcript to toggle a specific thinking/tool block (when not a drag).
    TranscriptClick {
        column: u16,
        row: u16,
    },
    /// Begin an in-app text selection drag at terminal-absolute coordinates.
    SelectionDragStart {
        column: u16,
        row: u16,
    },
    /// Update the live selection drag pointer.
    SelectionDragUpdate {
        column: u16,
        row: u16,
    },
    /// Finish a selection drag. If the pointer moved, the selected text is
    /// copied to the clipboard and the highlight stays until the next gesture;
    /// otherwise the original click is treated as a transcript toggle.
    SelectionDragEnd {
        column: u16,
        row: u16,
    },
    /// Clear any committed selection highlight.
    SelectionClear,
    StartRecoveryContinuation,
    /// Open the theme picker modal (Ctrl+T / `/theme`). Up/Down previews live;
    /// Enter persists; Esc restores the previous theme.
    CycleTheme,
    ThemePicker(ThemePickerAction),
    /// Cycle the session permission mode (Tab): normal → auto-edit → plan.
    CyclePermissionMode,
    /// Cycle model reasoning depth (Shift+Tab): minimal → low → medium → high → xhigh.
    CycleReasoningEffort,
    /// Begin reject-and-redirect on the focused approval: open the one-line reason
    /// capture in the bottom approval surface.
    ApprovalRejectBegin,
    /// Append a character to the in-progress reject reason.
    ApprovalRejectInput(char),
    /// Backspace in the in-progress reject reason.
    ApprovalRejectBackspace,
    /// Cancel reject-reason capture, leaving the request pending.
    ApprovalRejectCancel,
    /// Non-destructively dismiss the bottom approval surface: hide it but leave
    /// the request pending (the footer keeps showing the pending indicator).
    /// Never denies.
    ApprovalModalDismiss,
    /// Submit the captured reject reason as the deny reason.
    ApprovalRejectSubmit,
    /// Move the approval surface's highlighted option up one row.
    ApprovalSelectPrevious,
    /// Move the approval surface's highlighted option down one row.
    ApprovalSelectNext,
    /// Confirm the currently highlighted approval option (Enter).
    ApprovalConfirmSelection,
    /// Open the slash-command menu in place of a dedicated help panel.
    OpenCommands,
    OpenChat,
    Composer(ComposerAction),
    ApprovalPanel(ApprovalPanelAction),
    SessionPanel(SessionPanelAction),
    ModelPicker(ModelPickerAction),
    /// Dismiss the `/status` popup (Esc). Leaves the refreshed snapshot in state.
    StatusPanelDismiss,
    DiffPanel(DiffPanelAction),
    CommandPanel(CommandPanelAction),
    BridgeFrame(Box<UiJsonFrame>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiApp {
    pub screen: TuiScreen,
    pub status: StatusBarState,
    pub terminal_size: TerminalSize,
    pub theme: TerminalTheme,
    /// When false, model/reasoning bodies render as plain text (no Markdown).
    /// Defaults to true; set false via `RUSE_MARKDOWN=plain|0|off` at startup.
    pub markdown_enabled: bool,
    pub composer: ComposerState,
    pub transcript: TranscriptState,
    pub approval_panel: ApprovalPanelState,
    pub diff_panel: Option<DiffPanelState>,
    pub command_panel: Option<CommandPanelState>,
    pub trace_panel: TracePanelState,
    pub task_panel: TaskPanelState,
    pub extension_panel: ExtensionPanelState,
    pub session_panel: SessionPanelState,
    pub active_conversation_id: Option<ConversationId>,
    pub superseded_conversation_ids: BTreeSet<ConversationId>,
    pub conversation_replay_pending: bool,
    pub model_picker: ModelPickerState,
    pub theme_picker: ThemePickerState,
    pub status_panel: StatusPanelState,
    pub side_panel: TuiSidePanel,
    pub last_retry_candidate: Option<CommandRetryCandidateState>,
    pub pending_recovery_continuation: Option<RecoveryContinuationState>,
    pub copied_output: Option<String>,
    pub should_quit: bool,
    /// When the active turn began, if one is running. This is the authoritative
    /// "is a turn active" signal and the wall-clock origin for the elapsed-time
    /// readout. `None` means idle.
    pub turn_started_at: Option<Instant>,
    /// Index into the spinner frame set for the live activity indicator. The
    /// event loop recomputes this from elapsed wall-clock time each iteration so
    /// the animation cadence is independent of poll/redraw frequency and never
    /// freezes while the user types. Rendering reads it as pure state, so tests
    /// stay deterministic (default 0).
    pub spinner_frame: usize,
    /// Whole elapsed seconds since the active turn began, recomputed by the event
    /// loop from `turn_started_at` alongside `spinner_frame`. Rendered on the live
    /// activity line as pure state so the render layer never calls `Instant::now()`
    /// (keeping snapshots deterministic). 0 when idle.
    pub elapsed_seconds: u64,
    /// Whether a cancel has already been requested for the active turn. The first
    /// Ctrl+C requests cooperative cancellation; a second (while still active)
    /// escalates to force-quit. Reset when a turn starts or finishes so each turn
    /// gets a fresh two-press escape hatch.
    pub cancel_requested: bool,
    /// Identity (approval request id) of the approval the current `diff_panel`
    /// was derived from, when it came from a focused approval rather than an
    /// explicit `EditDiffSnapshot`. Used to avoid rebuilding the panel — and
    /// discarding its scroll/fold state — on every bridge frame.
    pub diff_panel_source: DiffPanelSource,
    /// Session-level permission mode, cycled with Tab and shown in the footer.
    /// Governs how much autonomy the agent has for tool actions.
    pub permission_mode: PermissionMode,
    /// In-memory, session-only scoped allow-always rules. When a pending approval
    /// matches a rule (same tool + resource prefix) it can be auto-allowed without
    /// prompting again. Cleared when the process exits — never persisted.
    pub allow_rules: Vec<AllowRule>,
    /// Active reject-and-redirect reason capture, if the user chose "Reject" on the
    /// bottom approval surface. While `Some`, that surface shows a one-line reason
    /// input; on submit the text becomes the deny reason (and the next steering
    /// message).
    pub reject_reason: Option<RejectReasonInput>,
    /// Whether the user pressed Esc to dismiss the bottom approval surface for the
    /// currently focused request, leaving it pending. The surface stays hidden
    /// (and normal keys resume) until a new/different request arrives, but the
    /// footer keeps showing the pending indicator so the gate is never
    /// armed-but-invisible. Reset when the focused request changes or the queue
    /// empties.
    pub approval_modal_dismissed: bool,
    /// A transient, neutral/dim toast for no-op guidance and success confirmations
    /// (clipboard copy, "nothing to cancel"). Shown briefly then auto-expired by
    /// the event loop; never a red error chip. `None` when no toast is showing.
    pub toast: Option<TuiToast>,
    /// Which approval option is highlighted (Up/Down selects, Enter confirms the
    /// highlighted one). Defaults to Allow once for the common approval path.
    /// Reset whenever the surface opens for a fresh request.
    pub approval_selection: ApprovalOption,
    /// In-app transcript selection (restores select+copy under mouse capture).
    pub selection: SelectionState,
    /// Latest pointer position in terminal-absolute coordinates. Used to
    /// brighten collapsible thinking headers on hover as a click affordance.
    pub pointer: Option<(u16, u16)>,
}

/// The three explicit approval choices, in display order. `Enter` confirms the
/// highlighted option, which defaults to `AllowOnce`; the digit keys `1`/`2`/`3`
/// still act directly regardless of the highlight.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ApprovalOption {
    /// Allow this request once.
    #[default]
    AllowOnce,
    /// Allow and record a session-scoped allow-always rule.
    AllowAlways,
    /// Reject and redirect.
    Reject,
}

impl ApprovalOption {
    /// The previous option (Up), saturating at the first.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::AllowOnce => Self::AllowOnce,
            Self::AllowAlways => Self::AllowOnce,
            Self::Reject => Self::AllowAlways,
        }
    }

    /// The next option (Down), saturating at the last.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::AllowOnce => Self::AllowAlways,
            Self::AllowAlways => Self::Reject,
            Self::Reject => Self::Reject,
        }
    }
}

/// How long a transient toast stays on screen before the event loop expires it.
pub const TOAST_TTL: Duration = Duration::from_secs(2);

/// A transient status note shown near the composer for no-op guidance and success
/// confirmations. Neutral/dim, never red. Auto-expires: the event loop clears it
/// once `shown_at` is older than `TOAST_TTL`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiToast {
    /// The message text.
    pub message: String,
    /// Whether this is a note or a (non-error) warning — both render neutral/dim,
    /// but a warning may be styled slightly more prominently. Never `Error` (real
    /// errors go to the transcript, not a toast).
    pub severity: LocalDiagnosticSeverity,
    /// When the toast was shown, used by the loop to expire it. `None` means it has
    /// no wall-clock origin yet (e.g. set in a pure test); such a toast does not
    /// auto-expire until a real `Instant` is attached.
    pub shown_at: Option<Instant>,
}

/// Session permission mode, cycled with Tab. Every mode is enforced by the
/// runtime policy used for the submitted turn.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum PermissionMode {
    /// Every side-effecting action prompts for approval.
    #[default]
    Normal,
    /// File edits apply without prompting; other actions still prompt.
    AutoAcceptEdits,
    /// Read-only: the agent may analyze and propose but not make changes.
    Plan,
    /// Registered side-effecting tools execute without interactive approval.
    Yolo,
}

impl PermissionMode {
    /// Cycle to the next mode (Tab): normal -> auto-edit -> plan -> yolo -> normal.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Normal => Self::AutoAcceptEdits,
            Self::AutoAcceptEdits => Self::Plan,
            Self::Plan => Self::Yolo,
            Self::Yolo => Self::Normal,
        }
    }

    /// Stable, user-facing label shown in the footer.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::AutoAcceptEdits => "auto-edit",
            Self::Plan => "plan",
            Self::Yolo => "yolo",
        }
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A session-only scoped allow-always rule. Matches a pending approval when the
/// tool/action kind is equal and the requested resource starts with `resource_prefix`.
/// Kept in memory on `TuiApp` and never persisted (session-only, per design).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowRule {
    /// The resource kind (tool class), e.g. `"command"` or `"file"`.
    pub resource_kind: String,
    /// The action, e.g. `"execute"` or `"write"`.
    pub action: String,
    /// Prefix the requested resource must start with for the rule to match. This is
    /// the "tool + resource prefix" scope: broad enough to kill repeat prompts for
    /// the same class of action, narrow enough not to blanket-approve everything.
    pub resource_prefix: String,
}

impl AllowRule {
    /// Whether this rule authorizes the given pending request.
    #[must_use]
    pub fn matches(&self, request: &UiApprovalRequest) -> bool {
        self.resource_kind == request.resource_kind
            && self.action == request.action
            && request.resource.starts_with(&self.resource_prefix)
    }
}

/// A one-line reason input shown in the bottom approval surface when the user
/// chooses "Reject & redirect". The captured text becomes the deny reason.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RejectReasonInput {
    /// The approval request id being rejected.
    pub request_id: String,
    /// The reason text typed so far.
    pub text: String,
}

/// Where the currently displayed `diff_panel` came from, so the reducer only
/// rebuilds it when the underlying source actually changes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DiffPanelSource {
    /// No diff panel is shown.
    #[default]
    None,
    /// Derived from a focused approval request with this id.
    Approval(String),
    /// Set explicitly from an `EditDiffSnapshot` frame.
    Snapshot,
}

/// Explicit, UI-facing lifecycle of the current turn. This is the single source
/// of turn truth the render layer consumes: whether a turn is running, and what
/// phase it is in. `Idle` means no turn is active.
///
/// The phase (`Waiting`/`Thinking`/`Responding`/`ToolRunning`) is derived from
/// the transcript's streaming state, but the *active vs idle* distinction comes
/// from the authoritative turn clock (`turn_started_at`), so a queued or
/// recovery-continuation turn reports `Waiting` (and shows the spinner) the
/// instant it starts — before any model frame arrives — instead of looking idle.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TurnState {
    /// No turn is running.
    Idle,
    /// A turn has started but no model/reasoning output has arrived yet.
    Waiting,
    /// The model is streaming reasoning/thinking output.
    Thinking,
    /// The model is streaming its final answer.
    Responding,
    /// A tool call is running.
    ToolRunning,
    /// A cancellation has been requested and we are waiting for the turn to stop.
    Cancelling,
}

impl TurnState {
    /// Whether a turn is active (anything other than `Idle`).
    #[must_use]
    pub fn is_active(self) -> bool {
        !matches!(self, TurnState::Idle)
    }

    /// Human-facing verb shown next to the spinner, or `None` when idle.
    #[must_use]
    pub fn label(self) -> Option<&'static str> {
        match self {
            TurnState::Idle => None,
            TurnState::Waiting => Some("Waiting"),
            TurnState::Thinking => Some("Thinking"),
            TurnState::Responding => Some("Responding"),
            TurnState::ToolRunning => Some("Working"),
            TurnState::Cancelling => Some("Cancelling"),
        }
    }
}

/// The connection/turn health surfaced in the persistent status footer and the
/// Status side panel. This is the single normalized vocabulary: every raw status
/// string that arrives from the runtime (model-attempt statuses, `TurnState`
/// debug strings like `"modelstreaming"`, error recoverability) is parsed into
/// one of these variants so a user never sees a raw `{:?}` Debug string or an
/// internal status code. Both the footer and the Status panel render through
/// `Display`, so they can never disagree.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ConnectionState {
    /// No session/turn activity yet.
    #[default]
    Disconnected,
    /// Idle but connected — a turn has completed and we are ready for input.
    Idle,
    /// A turn is actively streaming from the model/runtime.
    Streaming,
    /// The runtime reported a recoverable problem.
    Degraded,
    /// The runtime reported an unrecoverable failure.
    Failed,
}

impl ConnectionState {
    /// Parse any raw connection/status string the runtime emits into the
    /// normalized vocabulary. Known error/degraded/streaming/idle spellings map
    /// exactly; any unrecognized code maps to the safe neutral reading (`Idle`)
    /// so a stray internal string never masquerades as an error in the footer.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "disconnected" => Self::Disconnected,
            "idle" | "succeeded" | "completed" | "done" => Self::Idle,
            "failed" | "error" => Self::Failed,
            "degraded" | "retrying" | "rate_limited" | "timeout" => Self::Degraded,
            // Any streaming/running spelling, including raw `TurnState` debug
            // strings such as "modelstreaming"/"toolrunning"/"started".
            other if other.contains("stream") || other.contains("running") => Self::Streaming,
            "started" | "waiting_approval" | "streaming" | "thinking" | "responding"
            | "context-ready" => Self::Streaming,
            _ => Self::Idle,
        }
    }

    /// Stable, user-facing label. Never a raw code or Debug string.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Idle => "idle",
            Self::Streaming => "streaming",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl TuiApp {
    #[must_use]
    pub fn new(status: StatusBarState) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }

    /// Mark a turn as started, anchoring the wall-clock origin used by both the
    /// spinner animation and the elapsed-time readout. Idempotent within a turn:
    /// if a turn is already running the existing origin is kept so the elapsed
    /// clock does not reset when a queued/recovery turn chains off the previous.
    pub fn mark_turn_started(&mut self, now: Instant) {
        if self.turn_started_at.is_none() {
            self.turn_started_at = Some(now);
            self.spinner_frame = 0;
            self.elapsed_seconds = 0;
            self.cancel_requested = false;
        }
    }

    /// Mark the turn as finished (idle). Clears the turn clock and resets the
    /// spinner so the next turn animates from the first frame.
    pub fn mark_turn_finished(&mut self) {
        self.turn_started_at = None;
        self.spinner_frame = 0;
        self.elapsed_seconds = 0;
        self.cancel_requested = false;
    }

    /// Whether a turn is currently active (the authoritative signal, independent
    /// of transcript streaming heuristics).
    #[must_use]
    pub fn turn_active(&self) -> bool {
        self.turn_started_at.is_some()
    }

    /// Transcript item under the pointer, if any. Used to brighten collapsible
    /// thinking headers on hover.
    #[must_use]
    pub fn hovered_transcript_item(&self) -> Option<usize> {
        let (column, row) = self.pointer?;
        match crate::hit_test::hit_test_transcript(self, column, row) {
            Some(crate::TranscriptHit::ToggleReasoning { index })
            | Some(crate::TranscriptHit::ToggleToolOutput { index }) => Some(index),
            None => None,
        }
    }

    /// The explicit, UI-facing turn state. This is the single source of truth for
    /// "what is the turn doing right now", combining the authoritative turn clock
    /// (`turn_started_at`), the cancel flag, and the streaming-derived phase.
    ///
    /// When no turn is active it is `Idle`. When a turn is active but nothing has
    /// streamed yet — the common case for queued and recovery-continuation turns
    /// before the first delta arrives — it is `Waiting`, so the spinner shows
    /// immediately rather than only once the model starts producing output. Once
    /// the transcript begins streaming, the phase (`Thinking`/`Responding`/
    /// `ToolRunning`) is derived from the streaming item. A pending cancel wins
    /// over everything else.
    #[must_use]
    pub fn turn_state(&self) -> TurnState {
        if !self.turn_active() {
            return TurnState::Idle;
        }
        if self.cancel_requested {
            return TurnState::Cancelling;
        }
        match self.transcript.waiting_activity() {
            None => TurnState::Waiting,
            Some(WaitingActivity::Waiting) => TurnState::Waiting,
            Some(WaitingActivity::Thinking) => TurnState::Thinking,
            Some(WaitingActivity::Responding) => TurnState::Responding,
            Some(WaitingActivity::Working) => TurnState::ToolRunning,
        }
    }

    /// Whether an approval decision is pending focus (regardless of Esc dismiss).
    /// Render/key-routing use [`Self::approval_modal_visible`] for the live surface.
    #[must_use]
    pub fn approval_modal_active(&self) -> bool {
        self.approval_panel.focused_request().is_some()
    }

    /// Number of pending approvals, for the persistent footer indicator.
    #[must_use]
    pub fn pending_approval_count(&self) -> usize {
        self.approval_panel.pending.len()
    }

    /// Show a transient toast (destination (4) of the noise taxonomy): neutral/dim
    /// no-op guidance or a success confirmation. `now` anchors auto-expiry. Only
    /// `Note`/`Warning` severities reach here; errors go to the transcript instead.
    pub fn show_toast(
        &mut self,
        message: impl Into<String>,
        severity: LocalDiagnosticSeverity,
        now: Instant,
    ) {
        self.toast = Some(TuiToast {
            message: message.into(),
            severity,
            shown_at: Some(now),
        });
    }

    /// Expire the toast if it has been showing longer than `TOAST_TTL`. Returns
    /// whether anything changed (a redraw hint). Called by the event loop each tick.
    /// A toast with no `shown_at` (set in a pure test) never auto-expires.
    pub fn expire_toast(&mut self, now: Instant) -> bool {
        let expired = self.toast.as_ref().is_some_and(|toast| {
            toast
                .shown_at
                .is_some_and(|shown_at| now.saturating_duration_since(shown_at) >= TOAST_TTL)
        });
        if expired {
            self.toast = None;
        }
        expired
    }

    /// Whether the bottom-docked approval surface should currently replace the
    /// composer (Claude Code style). True when a pending request is focused and
    /// the user has not dismissed it with Esc. Dismissal is non-destructive: the
    /// request stays pending and the footer keeps showing the pending indicator,
    /// but the surface steps aside so the user can read the transcript or type —
    /// reopened by the next request or by focusing the approval panel.
    #[must_use]
    pub fn approval_modal_visible(&self) -> bool {
        self.approval_panel.focused_request().is_some() && !self.approval_modal_dismissed
    }

    /// Cycle the session permission mode (Tab): normal → auto-accept-edits → plan
    /// → normal. The active mode is always shown in the footer.
    pub fn cycle_permission_mode(&mut self) {
        self.permission_mode = self.permission_mode.next();
    }

    /// Handle an approval action, applying the `TuiApp`-level state that the pure
    /// `ApprovalPanelState` cannot reach: recording a session-scoped allow-always
    /// rule, and opening/closing the reject-reason capture. Everything else is
    /// forwarded to the panel reducer.
    fn handle_approval_action(&mut self, action: ApprovalPanelAction) -> TuiEffect {
        let effect = match action {
            ApprovalPanelAction::AllowAlways => {
                // Record a session-scoped rule (tool + resource prefix) so matching
                // future requests auto-approve, then allow this one once.
                if let Some(request) = self.approval_panel.focused_request() {
                    let rule = AllowRule {
                        resource_kind: request.resource_kind.clone(),
                        action: request.action.clone(),
                        resource_prefix: request.resource.clone(),
                    };
                    if !self.allow_rules.contains(&rule) {
                        self.allow_rules.push(rule);
                    }
                }
                self.approval_panel.reduce(ApprovalPanelAction::Allow)
            }
            other => self.approval_panel.reduce(other),
        };

        // Hide the modal as soon as a valid decision is submitted. The runtime
        // acknowledgement still removes the pending request from durable UI state,
        // but a potentially long continuation must not leave the overlay blocking
        // input while it runs.
        if matches!(effect, TuiEffect::ResolveApproval { .. }) {
            self.approval_modal_dismissed = true;
        }
        effect
    }

    /// Whether a pending request is already authorized by a session allow-rule.
    #[must_use]
    pub fn request_is_pre_authorized(&self, request: &UiApprovalRequest) -> bool {
        self.allow_rules.iter().any(|rule| rule.matches(request))
    }

    /// Elapsed wall-clock time since the active turn began, if any.
    #[must_use]
    pub fn turn_elapsed(&self, now: Instant) -> Option<Duration> {
        self.turn_started_at
            .map(|started| now.saturating_duration_since(started))
    }

    /// Recompute the spinner frame index from elapsed wall-clock time. Called by
    /// the event loop each iteration so the animation cadence is independent of
    /// poll/redraw frequency and advances even while the user is typing. Returns
    /// whether the frame index changed (a redraw hint). No-op when idle.
    pub fn refresh_spinner(&mut self, now: Instant) -> bool {
        let Some(elapsed) = self.turn_elapsed(now) else {
            return false;
        };
        let next = (elapsed.as_millis() / SPINNER_FRAME_INTERVAL_MS) as usize % SPINNER_FRAME_COUNT;
        let next_seconds = elapsed.as_secs();
        let mut changed = next != self.spinner_frame || next_seconds != self.elapsed_seconds;
        self.spinner_frame = next;
        self.elapsed_seconds = next_seconds;
        // Live thinking headers (`[3s · 100 tokens]`) tick with the spinner so
        // the render layer stays pure (no Instant::now in paint).
        if self.transcript.refresh_thinking_stats(now) {
            changed = true;
        }
        changed
    }

    pub fn reduce(&mut self, action: TuiAction) -> TuiEffect {
        match action {
            TuiAction::Tick => {
                // The spinner is driven from wall-clock time by the event loop
                // (see `refresh_spinner`), not from tick counting, so a tick is
                // just a redraw nudge here.
                TuiEffect::None
            }
            TuiAction::QuitRequested => {
                self.should_quit = true;
                TuiEffect::None
            }
            TuiAction::CancelTurnRequested => TuiEffect::note("no running turn to cancel"),
            TuiAction::Resize { width, height } => {
                self.terminal_size = TerminalSize { width, height };
                TuiEffect::None
            }
            TuiAction::ClearTranscript => {
                self.transcript = TranscriptState::default();
                self.selection.clear();
                TuiEffect::None
            }
            TuiAction::ScrollTranscriptUp => {
                // Mouse-wheel notches scroll a few lines so the transcript moves at a
                // readable pace; PageUp uses ScrollTranscriptPageUp for full pages.
                // Transcript selections use absolute wrapped rows, so scrolling
                // preserves both committed highlights and an in-progress drag.
                self.transcript.scroll_up(self.transcript_scroll_step());
                TuiEffect::None
            }
            TuiAction::ScrollTranscriptDown => {
                self.transcript.scroll_down(self.transcript_scroll_step());
                TuiEffect::None
            }
            TuiAction::ScrollTranscriptPageUp => {
                self.transcript.scroll_up(self.transcript_page_lines());
                TuiEffect::None
            }
            TuiAction::ScrollTranscriptPageDown => {
                self.transcript.scroll_down(self.transcript_page_lines());
                TuiEffect::None
            }
            TuiAction::ScrollTranscriptTop => {
                self.transcript.scroll_to_top();
                TuiEffect::None
            }
            TuiAction::ScrollTranscriptBottom => {
                self.transcript.scroll_to_bottom();
                TuiEffect::None
            }
            TuiAction::ToggleToolOutput => {
                if self.transcript.toggle_latest_tool_output() {
                    TuiEffect::None
                } else {
                    TuiEffect::note("no tool output is available to expand")
                }
            }
            TuiAction::ToggleReasoningAt { index } => {
                let _ = crate::hit_test::toggle_reasoning_at(&mut self.transcript, index);
                TuiEffect::None
            }
            TuiAction::ToggleToolOutputAt { index } => {
                let _ = crate::hit_test::toggle_tool_output_at(&mut self.transcript, index);
                TuiEffect::None
            }
            TuiAction::PointerMove { column, row } => {
                self.pointer = Some((column, row));
                TuiEffect::None
            }
            TuiAction::TranscriptClick { column, row } => {
                // Ignore clicks while a modal owns the screen — they would
                // otherwise punch through to transcript items behind the overlay.
                if self.approval_modal_visible()
                    || self.model_picker.is_open()
                    || self.session_panel.is_open()
                    || self.theme_picker.is_open()
                    || self.status_panel.is_open()
                {
                    return TuiEffect::None;
                }
                self.selection.clear();
                match crate::hit_test::hit_test_transcript(self, column, row) {
                    Some(crate::TranscriptHit::ToggleReasoning { index }) => {
                        let _ = crate::hit_test::toggle_reasoning_at(&mut self.transcript, index);
                    }
                    Some(crate::TranscriptHit::ToggleToolOutput { index }) => {
                        let _ = crate::hit_test::toggle_tool_output_at(&mut self.transcript, index);
                    }
                    None => {}
                }
                TuiEffect::None
            }
            TuiAction::SelectionDragStart { column, row } => {
                if self.approval_modal_visible()
                    || self.model_picker.is_open()
                    || self.session_panel.is_open()
                    || self.theme_picker.is_open()
                    || self.status_panel.is_open()
                {
                    return TuiEffect::None;
                }
                // Bind the gesture to the surface it starts in: a drag begun
                // inside the composer's text area selects composer input, anything
                // else selects the transcript. Starting a new gesture clears any
                // previous committed highlight.
                let (region, geo) = match crate::selection::ContentGeometry::composer_from_app(self)
                {
                    Some(geo) if geo.contains(column, row) => {
                        (crate::selection::SelectionRegion::Composer, geo)
                    }
                    _ => {
                        let Some(geo) = crate::selection::ContentGeometry::from_app(self) else {
                            return TuiEffect::None;
                        };
                        (crate::selection::SelectionRegion::Transcript, geo)
                    }
                };
                let viewport_start = if region == crate::selection::SelectionRegion::Transcript {
                    crate::selection::transcript_viewport_start(self, &geo)
                } else {
                    0
                };
                self.selection
                    .begin_drag(column, row, region, &geo, viewport_start);
                TuiEffect::None
            }
            TuiAction::SelectionDragUpdate { column, row } => {
                self.selection.update_drag(column, row);
                TuiEffect::None
            }
            TuiAction::SelectionDragEnd { column, row } => {
                // Final pointer position may differ from the last drag update.
                self.selection.update_drag(column, row);
                // Map the pointer against the geometry of the region the gesture
                // began in, so a composer drag commits composer-local columns.
                let geo = match self.selection.region {
                    crate::selection::SelectionRegion::Composer => {
                        crate::selection::ContentGeometry::composer_from_app(self)
                    }
                    crate::selection::SelectionRegion::Transcript => {
                        crate::selection::ContentGeometry::from_app(self)
                    }
                };
                let Some(geo) = geo else {
                    self.selection.clear();
                    return TuiEffect::None;
                };
                let viewport_start =
                    if self.selection.region == crate::selection::SelectionRegion::Transcript {
                        crate::selection::transcript_viewport_start(self, &geo)
                    } else {
                        0
                    };
                if self.selection.end_drag(&geo, viewport_start) {
                    // Real drag: copy selected text on release. Keep the
                    // highlight until the next gesture so the user can see
                    // what was copied.
                    self.copy_selection()
                } else if matches!(
                    self.selection.region,
                    crate::selection::SelectionRegion::Transcript
                ) {
                    // Click (no drag) in the transcript: hit-test toggle
                    // thinking/tool blocks. A click in the composer is a no-op
                    // here (the caret is driven by the widget, not mouse).
                    self.reduce(TuiAction::TranscriptClick { column, row })
                } else {
                    TuiEffect::None
                }
            }
            TuiAction::SelectionClear => {
                self.selection.clear();
                TuiEffect::None
            }
            TuiAction::StartRecoveryContinuation => self.start_recovery_continuation(),
            TuiAction::CycleTheme => {
                // Open the theme picker modal; live preview happens on focus
                // moves, Enter persists, Esc restores the original.
                self.screen = TuiScreen::Chat;
                self.handle_theme_picker(ThemePickerAction::Open)
            }
            TuiAction::ThemePicker(action) => self.handle_theme_picker(action),
            TuiAction::TogglePanelExpansion => {
                if let Some(panel) = &mut self.diff_panel {
                    panel.reduce(DiffPanelAction::ToggleFold);
                } else if let Some(panel) = &mut self.command_panel {
                    panel.reduce(CommandPanelAction::ToggleFold);
                } else {
                    return TuiEffect::note("no expandable panel is active");
                }
                TuiEffect::None
            }
            TuiAction::ToggleReasoning => {
                if self.transcript.toggle_latest_reasoning() {
                    TuiEffect::None
                } else {
                    TuiEffect::note("no thinking block is available")
                }
            }
            TuiAction::OpenCommands => {
                self.screen = TuiScreen::Chat;
                self.composer.slash_suggestions = SlashSuggestionState::from_input("/", 0);
                TuiEffect::None
            }
            TuiAction::OpenChat => {
                self.screen = TuiScreen::Chat;
                self.side_panel = TuiSidePanel::Auto;
                TuiEffect::None
            }
            TuiAction::Composer(action) => {
                // Typing or caret motion in the composer dismisses any in-app
                // selection highlight (transcript or composer).
                self.selection.clear();
                let effect = self.composer.reduce(action);
                self.apply_effect(effect)
            }
            TuiAction::ApprovalPanel(action) => self.handle_approval_action(action),
            TuiAction::CyclePermissionMode => {
                self.permission_mode = self.permission_mode.next();
                TuiEffect::None
            }
            TuiAction::CycleReasoningEffort => {
                self.status.reasoning_effort = self.status.reasoning_effort.next();
                TuiEffect::None
            }
            TuiAction::ApprovalModalDismiss => {
                // Non-destructive: hide the modal but leave the request pending.
                // The footer keeps showing the pending indicator, and any new
                // request re-shows the modal (the flag is reset on arrival).
                self.approval_modal_dismissed = true;
                TuiEffect::None
            }
            TuiAction::ApprovalRejectBegin => {
                // Open the one-line reason capture for the focused approval.
                match self.approval_panel.focused_request() {
                    Some(request) => {
                        self.reject_reason = Some(RejectReasonInput {
                            request_id: request.request_id.clone(),
                            text: String::new(),
                        });
                        TuiEffect::None
                    }
                    None => TuiEffect::note("no approval is focused"),
                }
            }
            TuiAction::ApprovalRejectInput(character) => {
                if let Some(input) = &mut self.reject_reason {
                    input.text.push(character);
                }
                TuiEffect::None
            }
            TuiAction::ApprovalRejectBackspace => {
                if let Some(input) = &mut self.reject_reason {
                    input.text.pop();
                }
                TuiEffect::None
            }
            TuiAction::ApprovalRejectCancel => {
                // Non-destructive: close the reason input, leave the request pending.
                self.reject_reason = None;
                TuiEffect::None
            }
            TuiAction::ApprovalRejectSubmit => {
                // Submit the captured reason as the deny reason (and steering hint).
                let Some(input) = self.reject_reason.take() else {
                    return TuiEffect::None;
                };
                let reason = input.text.trim();
                let reason = if reason.is_empty() {
                    "rejected from TUI".to_string()
                } else {
                    reason.to_string()
                };
                self.handle_approval_action(ApprovalPanelAction::Deny { reason })
            }
            TuiAction::ApprovalSelectPrevious => {
                self.approval_selection = self.approval_selection.previous();
                TuiEffect::None
            }
            TuiAction::ApprovalSelectNext => {
                self.approval_selection = self.approval_selection.next();
                TuiEffect::None
            }
            TuiAction::ApprovalConfirmSelection => {
                // Enter confirms the highlighted option. Allow once is the default
                // common path; Reject still opens explicit reason capture.
                match self.approval_selection {
                    ApprovalOption::AllowOnce => {
                        self.handle_approval_action(ApprovalPanelAction::Allow)
                    }
                    ApprovalOption::AllowAlways => {
                        self.handle_approval_action(ApprovalPanelAction::AllowAlways)
                    }
                    ApprovalOption::Reject => {
                        // Open the reason-capture flow (same as pressing 3).
                        self.reduce(TuiAction::ApprovalRejectBegin)
                    }
                }
            }
            TuiAction::SessionPanel(action) => {
                let effect = self.session_panel.reduce(action);
                if matches!(
                    effect,
                    TuiEffect::SwitchSession { .. } | TuiEffect::NewSession
                ) {
                    if let Some(current) = self.active_conversation_id {
                        self.superseded_conversation_ids.insert(current);
                    }
                    match &effect {
                        TuiEffect::SwitchSession {
                            conversation_id, ..
                        } => {
                            self.superseded_conversation_ids.remove(conversation_id);
                            self.active_conversation_id = Some(*conversation_id);
                            self.conversation_replay_pending = true;
                        }
                        TuiEffect::NewSession => {
                            self.active_conversation_id = None;
                            self.conversation_replay_pending = false;
                        }
                        _ => {}
                    }
                    self.transcript = TranscriptState::default();
                    self.approval_panel = ApprovalPanelState::default();
                    self.approval_modal_dismissed = false;
                    self.diff_panel = None;
                    self.command_panel = None;
                    self.trace_panel = TracePanelState::default();
                    self.pending_recovery_continuation = None;
                    self.cancel_requested = false;
                    self.mark_turn_finished();
                }
                effect
            }
            TuiAction::ModelPicker(action) => self.model_picker.reduce(action),
            TuiAction::StatusPanelDismiss => {
                self.status_panel.close();
                TuiEffect::None
            }
            TuiAction::DiffPanel(action) => {
                if let Some(panel) = &mut self.diff_panel {
                    panel.reduce(action);
                }
                TuiEffect::None
            }
            TuiAction::CommandPanel(action) => {
                if let Some(panel) = &mut self.command_panel {
                    panel.reduce(action);
                }
                TuiEffect::None
            }
            TuiAction::BridgeFrame(frame) => {
                if let UiJsonFrame::Event(event) = frame.as_ref() {
                    if self
                        .superseded_conversation_ids
                        .contains(&event.conversation_id)
                    {
                        return TuiEffect::None;
                    }
                    if self.active_conversation_id.is_none() {
                        self.active_conversation_id = Some(event.conversation_id);
                    }
                }
                self.approval_panel.reduce_frame(&frame);
                // A newly-arrived approval request re-shows the modal even if the
                // user had dismissed a previous one with Esc, so a fresh gate is
                // never left invisible.
                if let UiJsonFrame::Event(event) = frame.as_ref() {
                    if matches!(event.payload, UiEventPayload::ApprovalRequested(_)) {
                        self.approval_modal_dismissed = false;
                        // Reset the highlight to Allow once for each fresh request.
                        self.approval_selection = ApprovalOption::default();
                    }
                }
                self.trace_panel.reduce_frame(&frame);
                self.task_panel.reduce_frame(&frame);
                self.extension_panel.reduce_frame(&frame);
                self.session_panel.reduce_frame(&frame);
                self.model_picker.reduce_frame(&frame);
                self.status_panel.reduce_frame(&frame);
                self.reduce_status_frame(&frame);
                if let UiJsonFrame::Admin(admin) = frame.as_ref() {
                    match admin.kind.as_str() {
                        "command_test_snapshot" => {
                            if let Ok(snapshot) =
                                admin.decode_as::<ruse_ui_bridge::UiCommandTestSnapshot>()
                            {
                                if let Some(candidate) =
                                    CommandRetryCandidateState::from_command_test_snapshot(
                                        &snapshot,
                                    )
                                {
                                    self.last_retry_candidate = Some(candidate);
                                }
                                self.command_panel =
                                    Some(CommandPanelState::from_snapshot(&snapshot, 800));
                            }
                        }
                        "recovery_resume" => {
                            if let Ok(report) = admin.decode_as::<ruse_ui_bridge::UiResumeReport>()
                            {
                                self.pending_recovery_continuation = report
                                    .report
                                    .continuation
                                    .as_ref()
                                    .map(RecoveryContinuationState::from_continuation);
                            }
                        }
                        _ => {}
                    }
                }
                self.sync_diff_panel(frame.as_ref());
                if frame_should_enter_transcript(frame.as_ref()) {
                    self.transcript.reduce_frame(*frame);
                }
                TuiEffect::None
            }
        }
    }

    fn apply_effect(&mut self, effect: TuiEffect) -> TuiEffect {
        let TuiEffect::RunSlashCommand(command) = effect else {
            return effect;
        };

        self.apply_slash_command(command)
    }

    fn apply_slash_command(&mut self, command: SlashCommand) -> TuiEffect {
        match command {
            SlashCommand::Help => self.reduce(TuiAction::OpenCommands),
            SlashCommand::Status => {
                self.screen = TuiScreen::Chat;
                // Refresh the snapshot, surface it as a centered popup (same shape
                // as the model/session/theme modals), and request a fresh recovery
                // status snapshot from the runtime to fill in live connection state.
                self.refresh_status_panel("status");
                self.status_panel.open();
                TuiEffect::RecoveryStatus { turn_id: None }
            }
            SlashCommand::Model => {
                self.screen = TuiScreen::Chat;
                self.model_picker.reduce(ModelPickerAction::Open)
            }
            SlashCommand::Permissions => {
                self.screen = TuiScreen::Chat;
                self.refresh_status_panel("permissions");
                TuiEffect::note(format!(
                    "permission mode: {} · Tab to cycle",
                    self.permission_mode.label()
                ))
            }
            SlashCommand::Plan => {
                self.screen = TuiScreen::Chat;
                TuiEffect::note("plan/tasks live in the transcript and footer")
            }
            SlashCommand::Diff => {
                self.screen = TuiScreen::Chat;
                if self.diff_panel.is_none() {
                    TuiEffect::note("no diff is available yet")
                } else {
                    TuiEffect::note("diff is available for the focused approval")
                }
            }
            SlashCommand::Tools | SlashCommand::Mcp => {
                self.screen = TuiScreen::Chat;
                TuiEffect::note("tools/MCP status is shown inline during turns")
            }
            SlashCommand::Memory => {
                self.screen = TuiScreen::Chat;
                TuiEffect::note("memory status is shown inline during turns")
            }
            SlashCommand::Theme => {
                self.screen = TuiScreen::Chat;
                self.handle_theme_picker(ThemePickerAction::Open)
            }
            SlashCommand::Thinking(action) => {
                let (collapsed, label) = match action {
                    ThinkingVisibilityCommand::ShowAll | ThinkingVisibilityCommand::Show => {
                        (false, "expanded")
                    }
                    ThinkingVisibilityCommand::HideAll | ThinkingVisibilityCommand::Hide => {
                        (true, "collapsed")
                    }
                    ThinkingVisibilityCommand::Toggle => {
                        // Toggle defaults to expand when mixed/unknown.
                        (false, "expanded")
                    }
                };
                let count = self.transcript.set_all_reasoning_collapsed(collapsed);
                TuiEffect::note(format!("{count} thinking blocks {label}"))
            }
            SlashCommand::Compact => TuiEffect::ConversationCompact {
                max_summary_tokens: DEFAULT_COMPACTION_SUMMARY_TOKENS,
            },
            SlashCommand::Retry => {
                let has_evidence = self.last_retry_candidate.is_some();
                let (command_run_id, reason) = self.last_retry_candidate.as_ref().map_or_else(
                    || {
                        (
                            "missing-command-evidence".to_string(),
                            "requested from TUI /retry without command evidence".to_string(),
                        )
                    },
                    |candidate| {
                        (
                            candidate.command_run_id.clone(),
                            format!(
                                "requested from TUI /retry after {} {}",
                                candidate.status, candidate.classification
                            ),
                        )
                    },
                );
                TuiEffect::CommandRetryPlan {
                    command_run_id,
                    reason,
                    dry_run: !has_evidence,
                    evidence: self
                        .last_retry_candidate
                        .as_ref()
                        .map(|candidate| Box::new(candidate.evidence.clone())),
                }
            }
            SlashCommand::Trace => {
                self.screen = TuiScreen::Chat;
                TuiEffect::note("trace events stream into the transcript")
            }
            SlashCommand::Tasks => {
                self.screen = TuiScreen::Chat;
                TuiEffect::note("tasks live in the transcript and footer")
            }
            SlashCommand::Sessions => {
                // `/sessions` opens the session picker as a centered modal (like
                // the model picker), rather than a side panel.
                self.screen = TuiScreen::Chat;
                self.session_panel.reduce(SessionPanelAction::Open)
            }
            SlashCommand::New => {
                // `/new` starts a fresh session for this workspace. Reuse the
                // session-panel NewSession path so transcript/state cleanup stays
                // shared with the picker flow.
                self.screen = TuiScreen::Chat;
                self.reduce(TuiAction::SessionPanel(SessionPanelAction::NewSession))
            }
            SlashCommand::Resume => {
                self.screen = TuiScreen::Chat;
                TuiEffect::RecoveryResume {
                    turn_id: None,
                    checkpoint_id: None,
                }
            }
            SlashCommand::Settings => {
                TuiEffect::note("settings panel is not available in this milestone")
            }
            SlashCommand::Clear => {
                self.transcript = TranscriptState::default();
                TuiEffect::None
            }
            SlashCommand::Quit => {
                self.should_quit = true;
                TuiEffect::None
            }
        }
    }

    fn copy_selection(&mut self) -> TuiEffect {
        let Some(range) = self.selection.committed else {
            return TuiEffect::None;
        };
        // Dispatch on the gesture's starting surface so a composer drag never
        // pulls transcript text (and vice versa).
        let text = crate::selection::extract_selection(self, range, self.selection.region);
        if text.trim().is_empty() {
            // Empty range after normalize — drop the highlight, no clipboard write.
            self.selection.clear();
            return TuiEffect::None;
        }
        self.copied_output = Some(text.clone());
        TuiEffect::CopyToClipboard { text }
    }

    fn start_recovery_continuation(&mut self) -> TuiEffect {
        let Some(continuation) = self.pending_recovery_continuation.clone() else {
            return TuiEffect::note("no recovery continuation is available; run /resume first");
        };
        if !continuation.can_start_turn {
            return TuiEffect::note(format!(
                "recovery continuation is waiting on runtime state: {}",
                continuation.summary
            ));
        }

        self.pending_recovery_continuation = None;
        TuiEffect::SubmitRecoveryContinuation {
            prompt: continuation.next_prompt,
            summary: continuation.summary,
        }
    }

    fn transcript_page_lines(&self) -> u16 {
        self.terminal_size.height.saturating_sub(6).max(3)
    }

    /// Lines advanced per mouse-wheel notch. Kept small so continuous scrolling
    /// feels controllable; PageUp/PageDown jump by `transcript_page_lines`.
    fn transcript_scroll_step(&self) -> u16 {
        3
    }

    fn handle_theme_picker(&mut self, action: ThemePickerAction) -> TuiEffect {
        // Capture original before the reduce mutates picker state, so Esc can
        // restore it after a live preview.
        let original = self.theme_picker.original_theme;
        let effect = self.theme_picker.reduce(action.clone(), self.theme);
        match action {
            ThemePickerAction::Open => {
                // Opening focuses the current theme; nothing to preview yet.
            }
            ThemePickerAction::FocusNext | ThemePickerAction::FocusPrevious => {
                // Live preview: apply the focused theme immediately so the
                // whole UI recolors under the highlight.
                self.theme = self.theme_picker.focused_theme();
                self.refresh_status_panel("theme");
            }
            ThemePickerAction::Confirm => {
                self.theme = self.theme_picker.focused_theme();
                self.refresh_status_panel("theme");
            }
            ThemePickerAction::Dismiss => {
                // Cancelled: put the pre-open theme back so a half-previewed
                // theme never sticks without Enter.
                self.theme = original;
                self.refresh_status_panel("theme");
            }
        }
        effect
    }

    pub(crate) fn refresh_status_panel(&mut self, focused_panel: &str) {
        self.status_panel = StatusPanelState::from_status_bar_with_theme(
            &self.status,
            focused_panel,
            self.session_panel.sessions.len(),
            self.session_panel.diagnostics.len(),
            self.theme.name(),
        );
    }

    /// Keep `diff_panel` in sync without rebuilding it on every bridge frame.
    ///
    /// Only frames that can actually change the diff touch the panel:
    ///   - an explicit `EditDiffSnapshot` always replaces it, and
    ///   - an approval frame (which may have changed the focused request)
    ///     re-derives it, but ONLY when the focused request's identity actually
    ///     changes from what the panel was last built from.
    ///
    /// Every other frame — the many `ModelDelta`/`ToolObservation`/`ContextStatus`
    /// frames a streaming turn emits per second — is ignored here, so the user's
    /// scroll/fold position survives. Rebuilding on every frame previously reset
    /// that state and made the panel unusable mid-turn.
    fn sync_diff_panel(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::EditDiffSnapshot(snapshot) => {
                self.diff_panel = Some(DiffPanelState::from_edit_diff_snapshot(snapshot, 800));
                self.diff_panel_source = DiffPanelSource::Snapshot;
            }
            UiJsonFrame::ApprovalQueueSnapshot(_) => self.sync_diff_panel_from_focus(),
            UiJsonFrame::Event(event)
                if matches!(
                    event.payload,
                    UiEventPayload::ApprovalRequested(_) | UiEventPayload::ApprovalResolved { .. }
                ) =>
            {
                self.sync_diff_panel_from_focus();
            }
            _ => {}
        }
    }

    /// Re-derive the diff panel from the focused approval, but only when the
    /// focused request's identity differs from what the panel currently shows,
    /// so navigation state is preserved across redundant refreshes.
    fn sync_diff_panel_from_focus(&mut self) {
        let focused_source = self
            .approval_panel
            .focused_request()
            .map(|request| DiffPanelSource::Approval(request.request_id.clone()))
            .unwrap_or(DiffPanelSource::None);

        if focused_source == self.diff_panel_source {
            return;
        }

        self.diff_panel = self
            .approval_panel
            .focused_request()
            .and_then(|request| DiffPanelState::maybe_from_approval(request, 800));
        self.diff_panel_source = focused_source;
    }

    fn reduce_status_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::ModelSelection { provider, model } => {
                self.status.provider = provider.clone();
                self.status.model = model.clone();
                self.status.connection = "streaming".to_string();
            }
            UiJsonFrame::ConfigSummary {
                selected_provider,
                selected_model,
                ..
            } => {
                self.status.provider = selected_provider.clone();
                self.status.model = selected_model.clone();
            }
            UiJsonFrame::ConfigSnapshot(snapshot) => {
                self.status.provider = snapshot.selected_provider.clone();
                self.status.model = snapshot.selected_model.clone();
                self.status.context.total_tokens = snapshot
                    .providers
                    .iter()
                    .find(|provider| provider.id == snapshot.selected_provider)
                    .and_then(|provider| provider.capabilities.max_context_tokens);
            }
            UiJsonFrame::ModelInvocationAttempts(snapshot) => {
                self.status.connection = snapshot.status.clone();
            }
            UiJsonFrame::Done { .. } => {
                if self.status.connection == "streaming" {
                    self.status.connection = "idle".to_string();
                }
            }
            UiJsonFrame::Event(event) => match &event.payload {
                UiEventPayload::ContextStatus {
                    estimated_tokens, ..
                } => {
                    self.status.context.used_tokens = Some(*estimated_tokens);
                }
                UiEventPayload::TurnState { state, .. } => {
                    self.status.connection = format!("{state:?}").to_ascii_lowercase();
                }
                _ => {}
            },
            UiJsonFrame::Error { recoverable, .. } => {
                self.status.connection = if *recoverable {
                    "degraded".to_string()
                } else {
                    "failed".to_string()
                };
            }
            _ => {}
        }
    }
}

impl From<ComposerAction> for TuiAction {
    fn from(value: ComposerAction) -> Self {
        Self::Composer(value)
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            screen: TuiScreen::Chat,
            status: StatusBarState::default(),
            terminal_size: TerminalSize::default(),
            theme: TerminalTheme::default(),
            markdown_enabled: ruse_markdown::markdown_enabled_from_env(),
            composer: ComposerState::default(),
            transcript: TranscriptState::default(),
            approval_panel: ApprovalPanelState::default(),
            diff_panel: None,
            command_panel: None,
            trace_panel: TracePanelState::default(),
            task_panel: TaskPanelState::default(),
            extension_panel: ExtensionPanelState::default(),
            session_panel: SessionPanelState::default(),
            active_conversation_id: None,
            superseded_conversation_ids: BTreeSet::new(),
            conversation_replay_pending: false,
            model_picker: ModelPickerState::default(),
            theme_picker: ThemePickerState::default(),
            status_panel: StatusPanelState::default(),
            side_panel: TuiSidePanel::default(),
            last_retry_candidate: None,
            pending_recovery_continuation: None,
            copied_output: None,
            should_quit: false,
            turn_started_at: None,
            spinner_frame: 0,
            elapsed_seconds: 0,
            diff_panel_source: DiffPanelSource::default(),
            cancel_requested: false,
            permission_mode: PermissionMode::default(),
            allow_rules: Vec::new(),
            reject_reason: None,
            approval_modal_dismissed: false,
            approval_selection: ApprovalOption::default(),
            toast: None,
            selection: SelectionState::default(),
            pointer: None,
        }
    }
}

fn is_retry_candidate(snapshot: &UiCommandTestSnapshot) -> bool {
    if matches!(snapshot.status.as_str(), "succeeded") {
        return false;
    }

    matches!(
        snapshot.status.as_str(),
        "failed" | "timed_out" | "tool_failed" | "waiting_approval" | "denied"
    ) || matches!(
        snapshot.classification.as_str(),
        "test_failure"
            | "command_failure"
            | "timeout"
            | "tool_failure"
            | "waiting_approval"
            | "policy_denied"
            | "invalid_input"
    )
}

fn frame_should_enter_transcript(frame: &UiJsonFrame) -> bool {
    // Management/admin UiJsonFrame variants (project context, team, memory,
    // recovery reports, …) never enter the chat transcript. Core event
    // payloads still filter to chat-relevant kinds only.
    match frame {
        UiJsonFrame::Event(event) => matches!(
            event.payload,
            UiEventPayload::UserMessage { .. }
                | UiEventPayload::ReasoningSummaryDelta { .. }
                | UiEventPayload::ModelDelta { .. }
                | UiEventPayload::FinalResponse { .. }
                | UiEventPayload::ToolCall { .. }
                | UiEventPayload::ToolStarted { .. }
                | UiEventPayload::ToolObservation { .. }
                | UiEventPayload::ToolResult { .. }
                | UiEventPayload::ApprovalRequested(_)
                | UiEventPayload::ApprovalResolved { .. }
                | UiEventPayload::Diagnostic { .. }
        ),
        UiJsonFrame::Error { .. } => true,
        UiJsonFrame::ConfigDiagnostic { severity, .. } => severity != "info",
        other => {
            // Explicitly reject non-core snapshots so frame expansion cannot
            // accidentally bleed into the daily-driver transcript.
            let _ = ruse_ui_bridge::is_core_ui_frame(other);
            false
        }
    }
}

pub(crate) fn format_token_count(value: u32) -> String {
    if value >= 1_000_000 {
        format!("{}m", value / 1_000_000)
    } else if value >= 10_000 {
        format!("{}k", value / 1_000)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod context_window_tests {
    use super::*;

    #[test]
    fn render_falls_back_to_pending_unknown() {
        assert_eq!(ContextWindowState::default().render(), "pending/unknown");
        assert_eq!(
            ContextWindowState::pending(Some(128_000)).render(),
            "pending/128k"
        );
    }

    #[test]
    fn render_bar_falls_back_to_counts_when_usage_unknown() {
        // No used tokens yet — show the plain counts rather than an empty bar.
        assert_eq!(
            ContextWindowState::pending(Some(128_000)).render_bar(),
            "pending/128k"
        );
        assert_eq!(
            ContextWindowState::default().render_bar(),
            "pending/unknown"
        );
        assert!(ContextWindowState::default().usage_percent().is_none());
    }

    #[test]
    fn render_bar_draws_compact_meter_with_token_counts() {
        let half = ContextWindowState {
            used_tokens: Some(64_000),
            total_tokens: Some(128_000),
        };
        assert_eq!(half.usage_percent(), Some(50));
        // Bar + original used/total counts (not a bare percent).
        assert_eq!(half.render_bar(), "████░░░░ 64k/128k");

        let empty = ContextWindowState {
            used_tokens: Some(0),
            total_tokens: Some(128_000),
        };
        assert_eq!(empty.render_bar(), "░░░░░░░░ 0/128k");

        let full = ContextWindowState {
            used_tokens: Some(128_000),
            total_tokens: Some(128_000),
        };
        assert_eq!(full.render_bar(), "████████ 128k/128k");

        // Over-budget estimates clamp the bar to full; counts stay honest.
        let over = ContextWindowState {
            used_tokens: Some(200_000),
            total_tokens: Some(128_000),
        };
        assert_eq!(over.usage_percent(), Some(100));
        assert_eq!(over.render_bar(), "████████ 200k/128k");
    }
}

#[cfg(test)]
mod turn_clock_tests {
    use super::*;
    use crate::{TranscriptItem, TranscriptItemKind};

    #[test]
    fn turn_clock_starts_and_finishes() {
        let mut app = TuiApp::default();
        assert!(!app.turn_active(), "fresh app is idle");

        let start = Instant::now();
        app.mark_turn_started(start);
        assert!(app.turn_active(), "turn is active after start");
        assert_eq!(app.spinner_frame, 0);

        app.mark_turn_finished();
        assert!(!app.turn_active(), "turn is idle after finish");
        assert_eq!(app.spinner_frame, 0, "spinner resets on finish");
    }

    #[test]
    fn mark_turn_started_is_idempotent_and_keeps_origin() {
        let mut app = TuiApp::default();
        let origin = Instant::now();
        app.mark_turn_started(origin);
        let recorded = app.turn_started_at;

        // A queued/recovery turn chaining off the previous must not reset the
        // elapsed clock.
        app.mark_turn_started(origin + Duration::from_secs(5));
        assert_eq!(app.turn_started_at, recorded, "origin is preserved");
    }

    #[test]
    fn spinner_frame_advances_from_wall_clock_independent_of_calls() {
        let mut app = TuiApp::default();
        let start = Instant::now();
        app.mark_turn_started(start);

        // One frame interval elapsed advances exactly one frame.
        let changed =
            app.refresh_spinner(start + Duration::from_millis(SPINNER_FRAME_INTERVAL_MS as u64));
        assert!(changed, "frame advanced after one interval");
        assert_eq!(app.spinner_frame, 1);

        // Re-refreshing at the same instant is a no-op (no redundant redraw).
        let changed_again =
            app.refresh_spinner(start + Duration::from_millis(SPINNER_FRAME_INTERVAL_MS as u64));
        assert!(!changed_again, "no change without elapsed time");
        assert_eq!(app.spinner_frame, 1);

        // The frame index is a pure function of elapsed time, so it advances even
        // if refresh was not called for several intervals (e.g. the user was
        // typing and only key events fired) — it never freezes.
        app.refresh_spinner(start + Duration::from_millis(SPINNER_FRAME_INTERVAL_MS as u64 * 5));
        assert_eq!(app.spinner_frame, 5 % SPINNER_FRAME_COUNT);
    }

    #[test]
    fn refresh_spinner_is_noop_when_idle() {
        let mut app = TuiApp::default();
        assert!(!app.refresh_spinner(Instant::now()), "idle: no animation");
        assert_eq!(app.spinner_frame, 0);
    }

    #[test]
    fn spinner_advances_while_typing_during_a_turn() {
        // Regression: the spinner must keep animating while the user types. Because
        // the frame index is derived from the wall clock (not from a per-redraw
        // tick), composer input between refreshes neither resets nor freezes it.
        let mut app = TuiApp::default();
        let start = Instant::now();
        app.mark_turn_started(start);

        // The user types several characters; each is a composer mutation but must
        // not touch the turn clock or the spinner origin.
        app.reduce(TuiAction::Composer(ComposerAction::InsertChar('h')));
        app.reduce(TuiAction::Composer(ComposerAction::InsertChar('i')));
        assert_eq!(app.composer.text(), "hi");
        assert_eq!(
            app.turn_started_at,
            Some(start),
            "typing must not reset the turn clock"
        );

        // Time advances by one frame interval while typing continued; the next
        // refresh (driven by the loop) reflects the true elapsed frame, not frame 0.
        let later = start + Duration::from_millis(SPINNER_FRAME_INTERVAL_MS as u64);
        assert!(
            app.refresh_spinner(later),
            "spinner advanced despite typing"
        );
        assert_eq!(app.spinner_frame, 1 % SPINNER_FRAME_COUNT);

        // More typing, more elapsed time — still advancing, never frozen.
        // Pick a step that is not congruent mod SPINNER_FRAME_COUNT so the
        // frame index actually changes (6-frame star spinner wraps at 6).
        app.reduce(TuiAction::Composer(ComposerAction::InsertChar('!')));
        let later = start + Duration::from_millis(SPINNER_FRAME_INTERVAL_MS as u64 * 3);
        assert!(app.refresh_spinner(later), "spinner keeps advancing");
        assert_eq!(app.spinner_frame, 3 % SPINNER_FRAME_COUNT);
    }

    #[test]
    fn turn_elapsed_reports_wall_clock_since_start() {
        let mut app = TuiApp::default();
        assert!(
            app.turn_elapsed(Instant::now()).is_none(),
            "idle: no elapsed"
        );

        let start = Instant::now();
        app.mark_turn_started(start);
        let elapsed = app
            .turn_elapsed(start + Duration::from_secs(3))
            .expect("active turn has elapsed");
        assert_eq!(elapsed.as_secs(), 3);
    }

    #[test]
    fn turn_state_is_idle_when_no_turn_is_active() {
        let app = TuiApp::default();
        assert_eq!(app.turn_state(), TurnState::Idle);
        assert!(!app.turn_state().is_active());
        assert_eq!(app.turn_state().label(), None);
    }

    #[test]
    fn turn_state_reports_waiting_before_any_stream_arrives() {
        // A queued/recovery turn starts the clock but pushes no streaming item.
        // It must report an active, spinner-showing state immediately rather than
        // looking idle until the first model frame.
        let mut app = TuiApp::default();
        app.mark_turn_started(Instant::now());

        assert_eq!(app.turn_state(), TurnState::Waiting);
        assert!(app.turn_state().is_active());
        assert_eq!(app.turn_state().label(), Some("Waiting"));
    }

    #[test]
    fn turn_state_reflects_streaming_phase() {
        let mut app = TuiApp::default();
        app.mark_turn_started(Instant::now());

        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::Reasoning,
            "thinking",
            "reasoning...",
            true,
        ));
        assert_eq!(app.turn_state(), TurnState::Thinking);

        app.transcript = TranscriptState::default();
        app.transcript
            .push(TranscriptItem::streaming_model("answering..."));
        assert_eq!(app.turn_state(), TurnState::Responding);
    }

    #[test]
    fn turn_state_cancelling_wins_over_streaming_phase() {
        let mut app = TuiApp::default();
        app.mark_turn_started(Instant::now());
        app.transcript
            .push(TranscriptItem::streaming_model("answering..."));
        app.cancel_requested = true;

        assert_eq!(app.turn_state(), TurnState::Cancelling);
        assert_eq!(app.turn_state().label(), Some("Cancelling"));
    }

    #[test]
    fn switching_sessions_clears_conversation_state_and_ignores_old_late_frames() {
        let workspace_id = ruse_types::WorkspaceId::new();
        let old_conversation_id = ruse_types::ConversationId::new();
        let conversation_id = ruse_types::ConversationId::new();
        let mut app = TuiApp {
            active_conversation_id: Some(old_conversation_id),
            ..TuiApp::default()
        };
        app.reduce(TuiAction::BridgeFrame(Box::new(
            UiJsonFrame::TerminalSessionSnapshot(ruse_ui_bridge::UiTerminalSessionSnapshot {
                sessions: vec![ruse_ui_bridge::UiTerminalSessionSummary {
                    workspace_id,
                    conversation_id,
                    workspace_root_display: "workspace".to_string(),
                    title: "target".to_string(),
                    client_kind: "tui".to_string(),
                    client_version: "test".to_string(),
                    provider_id: "fake".to_string(),
                    model_id: "fake-scripted".to_string(),
                    git_state: "clean".to_string(),
                    last_event_at: ruse_types::Timestamp::now_utc(),
                    // Non-idle so the session picker does not skip this as an empty current
                    // conversation when choosing the default focus.
                    status: "has_tasks".to_string(),
                    pending_approval_count: 0,
                    running_command_count: 0,
                    task_count: 1,
                }],
                diagnostics: Vec::new(),
            }),
        )));
        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::User,
            "user",
            "old conversation",
            false,
        ));
        app.mark_turn_started(Instant::now());
        app.reduce(TuiAction::SessionPanel(SessionPanelAction::Open));

        let effect = app.reduce(TuiAction::SessionPanel(SessionPanelAction::SwitchFocused));

        assert_eq!(
            effect,
            TuiEffect::SwitchSession {
                workspace_id,
                conversation_id,
            }
        );
        assert!(
            !app.session_panel.is_open(),
            "session panel closes after Enter switch"
        );
        assert!(app.transcript.items.is_empty());
        assert!(!app.turn_active());
        assert_eq!(app.active_conversation_id, Some(conversation_id));

        let message_frame = |conversation_id, message: &str| {
            UiJsonFrame::Event(ruse_ui_bridge::UiEventEnvelope {
                schema_version: ruse_types::SchemaVersion::current(),
                event_id: ruse_types::EventId::new(),
                seq: None,
                occurred_at: ruse_types::Timestamp::now_utc(),
                conversation_id,
                turn_id: None,
                task_id: None,
                correlation_id: ruse_types::CorrelationId::new(),
                payload: UiEventPayload::UserMessage {
                    message: message.to_string(),
                },
            })
        };
        app.reduce(TuiAction::BridgeFrame(Box::new(message_frame(
            old_conversation_id,
            "late old message",
        ))));
        assert!(app.transcript.items.is_empty());

        app.reduce(TuiAction::BridgeFrame(Box::new(message_frame(
            conversation_id,
            "restored target message",
        ))));
        assert_eq!(app.transcript.items.len(), 1);
        assert_eq!(app.transcript.items[0].body, "restored target message");
    }
}

#[cfg(test)]
mod approval_modal_tests {
    use super::*;
    use ruse_ui_bridge::{
        UiApprovalPreviewSummary, UiApprovalRequest, UiEventEnvelope, UiEventPayload, UiJsonFrame,
    };

    fn command_request(id: &str, resource: &str) -> UiApprovalRequest {
        UiApprovalRequest {
            request_id: id.to_string(),
            status: "pending".to_string(),
            action: "execute".to_string(),
            resource_kind: "command".to_string(),
            resource: resource.to_string(),
            reason: "run verification".to_string(),
            preview: UiApprovalPreviewSummary {
                kind: "command".to_string(),
                resource: resource.to_string(),
                summary: format!("Run {resource}"),
                content_hash: None,
                truncated: false,
                redacted: false,
            },
            max_duration_seconds: None,
            expires_at: None,
        }
    }

    fn approval_frame(request: UiApprovalRequest) -> UiJsonFrame {
        UiJsonFrame::Event(UiEventEnvelope {
            schema_version: ruse_types::SchemaVersion::current(),
            event_id: ruse_types::EventId::new(),
            seq: None,
            occurred_at: ruse_types::Timestamp::now_utc(),
            conversation_id: ruse_types::ConversationId::new(),
            turn_id: None,
            task_id: None,
            correlation_id: ruse_types::CorrelationId::new(),
            payload: UiEventPayload::ApprovalRequested(Box::new(request)),
        })
    }

    fn push_request(app: &mut TuiApp, id: &str, resource: &str) {
        app.reduce(TuiAction::BridgeFrame(Box::new(approval_frame(
            command_request(id, resource),
        ))));
    }

    #[test]
    fn modal_is_visible_when_a_request_is_pending() {
        let mut app = TuiApp::default();
        assert!(!app.approval_modal_visible(), "idle: no modal");
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );
        assert!(app.approval_modal_visible(), "pending request shows modal");
    }

    #[test]
    fn esc_dismiss_is_non_destructive_and_leaves_request_pending() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );

        app.reduce(TuiAction::ApprovalModalDismiss);

        // The request is STILL pending (dismiss never denies), the modal is just
        // hidden, and the footer indicator remains.
        assert_eq!(app.pending_approval_count(), 1, "request stays pending");
        assert!(
            !app.approval_modal_visible(),
            "modal is hidden after dismiss"
        );
    }

    #[test]
    fn a_new_request_reopens_a_dismissed_modal() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );
        app.reduce(TuiAction::ApprovalModalDismiss);
        assert!(!app.approval_modal_visible());

        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000002",
            "cargo build",
        );
        assert!(
            app.approval_modal_visible(),
            "a fresh request re-shows the modal"
        );
    }

    #[test]
    fn allow_always_records_a_scoped_session_rule() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );

        let effect = app.reduce(TuiAction::ApprovalPanel(ApprovalPanelAction::AllowAlways));
        assert!(
            matches!(effect, TuiEffect::ResolveApproval { .. }),
            "allow-always still resolves this request as allow"
        );
        assert_eq!(app.allow_rules.len(), 1, "a scoped rule is recorded");

        // A matching future request (same tool + resource prefix) is pre-authorized.
        let matching = command_request("00000000-0000-0000-0000-000000000002", "cargo test");
        assert!(app.request_is_pre_authorized(&matching));
        // A non-matching request (different resource) is not.
        let other = command_request("00000000-0000-0000-0000-000000000003", "rm -rf /");
        assert!(!app.request_is_pre_authorized(&other));
    }

    #[test]
    fn reject_reason_capture_produces_a_deny_with_the_typed_reason() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );

        app.reduce(TuiAction::ApprovalRejectBegin);
        assert!(app.reject_reason.is_some(), "reason capture opened");

        for ch in "no thanks".chars() {
            app.reduce(TuiAction::ApprovalRejectInput(ch));
        }
        let effect = app.reduce(TuiAction::ApprovalRejectSubmit);
        match effect {
            TuiEffect::ResolveApproval {
                decision, reason, ..
            } => {
                assert_eq!(decision, ruse_ui_bridge::UiApprovalDecision::Deny);
                assert_eq!(reason, "no thanks");
            }
            other => panic!("expected a deny resolve, got {other:?}"),
        }
        assert!(app.reject_reason.is_none(), "capture closed after submit");
    }

    #[test]
    fn reject_reason_cancel_is_non_destructive() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );

        app.reduce(TuiAction::ApprovalRejectBegin);
        app.reduce(TuiAction::ApprovalRejectInput('x'));
        app.reduce(TuiAction::ApprovalRejectCancel);

        assert!(app.reject_reason.is_none(), "capture closed");
        assert_eq!(app.pending_approval_count(), 1, "request still pending");
    }

    #[test]
    fn cycle_permission_mode_walks_normal_auto_plan_yolo() {
        let mut app = TuiApp::default();
        assert_eq!(app.permission_mode, PermissionMode::Normal);
        app.reduce(TuiAction::CyclePermissionMode);
        assert_eq!(app.permission_mode, PermissionMode::AutoAcceptEdits);
        app.reduce(TuiAction::CyclePermissionMode);
        assert_eq!(app.permission_mode, PermissionMode::Plan);
        app.reduce(TuiAction::CyclePermissionMode);
        assert_eq!(app.permission_mode, PermissionMode::Yolo);
        app.reduce(TuiAction::CyclePermissionMode);
        assert_eq!(app.permission_mode, PermissionMode::Normal);
    }

    #[test]
    fn cycle_reasoning_effort_walks_all_five_depths() {
        let mut app = TuiApp::default();
        assert_eq!(app.status.reasoning_effort, ReasoningEffort::XHigh);
        for expected in [
            ReasoningEffort::Minimal,
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
        ] {
            app.reduce(TuiAction::CycleReasoningEffort);
            assert_eq!(app.status.reasoning_effort, expected);
        }
    }

    #[test]
    fn modal_selection_defaults_to_allow_once() {
        let app = TuiApp::default();
        assert_eq!(app.approval_selection, ApprovalOption::AllowOnce);
    }

    #[test]
    fn selection_moves_up_and_down_and_saturates() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );

        // Allow once is the top/default option and saturates upward.
        app.reduce(TuiAction::ApprovalSelectPrevious);
        assert_eq!(app.approval_selection, ApprovalOption::AllowOnce);

        app.reduce(TuiAction::ApprovalSelectNext);
        assert_eq!(app.approval_selection, ApprovalOption::AllowAlways);
        app.reduce(TuiAction::ApprovalSelectNext);
        assert_eq!(app.approval_selection, ApprovalOption::Reject);
        app.reduce(TuiAction::ApprovalSelectNext);
        assert_eq!(
            app.approval_selection,
            ApprovalOption::Reject,
            "saturates at bottom"
        );

        app.reduce(TuiAction::ApprovalSelectPrevious);
        assert_eq!(app.approval_selection, ApprovalOption::AllowAlways);
        app.reduce(TuiAction::ApprovalSelectPrevious);
        assert_eq!(app.approval_selection, ApprovalOption::AllowOnce);
    }

    #[test]
    fn confirming_default_selection_resolves_allow_once_and_hides_modal() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );

        let effect = app.reduce(TuiAction::ApprovalConfirmSelection);
        assert!(
            matches!(effect, TuiEffect::ResolveApproval { .. }),
            "confirming the default Allow once resolves approval, got {effect:?}"
        );
        assert!(
            !app.approval_modal_visible(),
            "modal hides while resolution runs"
        );
        assert_eq!(
            app.pending_approval_count(),
            1,
            "ack still owns durable removal"
        );
    }

    #[test]
    fn confirming_reject_selection_opens_reason_capture() {
        let mut app = TuiApp::default();
        push_request(
            &mut app,
            "00000000-0000-0000-0000-000000000001",
            "cargo test",
        );
        app.reduce(TuiAction::ApprovalSelectNext);
        app.reduce(TuiAction::ApprovalSelectNext);
        assert_eq!(app.approval_selection, ApprovalOption::Reject);

        let effect = app.reduce(TuiAction::ApprovalConfirmSelection);
        assert_eq!(effect, TuiEffect::None);
        assert!(app.reject_reason.is_some());
        assert!(app.approval_modal_visible());
    }
}

#[cfg(test)]
mod toast_tests {
    use super::*;

    #[test]
    fn show_toast_sets_neutral_note() {
        let mut app = TuiApp::default();
        let now = Instant::now();
        app.show_toast("copied to clipboard", LocalDiagnosticSeverity::Note, now);

        let toast = app.toast.as_ref().expect("toast is shown");
        assert_eq!(toast.message, "copied to clipboard");
        assert_eq!(toast.severity, LocalDiagnosticSeverity::Note);
    }

    #[test]
    fn toast_auto_expires_after_ttl() {
        let mut app = TuiApp::default();
        let now = Instant::now();
        app.show_toast("nothing to cancel", LocalDiagnosticSeverity::Note, now);

        // Before the TTL: no change, toast stays.
        assert!(!app.expire_toast(now + TOAST_TTL / 2));
        assert!(app.toast.is_some());

        // After the TTL: the loop clears it and reports a change (redraw hint).
        assert!(app.expire_toast(now + TOAST_TTL + Duration::from_millis(1)));
        assert!(app.toast.is_none());
    }

    #[test]
    fn note_and_error_diagnostics_carry_distinct_severity() {
        // The taxonomy split lives in the effect: notes are neutral (toast), only
        // errors are red (transcript). This asserts the constructor severity so the
        // routing in the effect handler can rely on it.
        assert!(matches!(
            TuiEffect::note("no-op guidance"),
            TuiEffect::LocalDiagnostic {
                severity: LocalDiagnosticSeverity::Note,
                ..
            }
        ));
        assert!(matches!(
            TuiEffect::error_diagnostic("a real failure"),
            TuiEffect::LocalDiagnostic {
                severity: LocalDiagnosticSeverity::Error,
                ..
            }
        ));
    }
}
