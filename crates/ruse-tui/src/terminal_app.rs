use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::{
    env,
    process::{Command, Stdio},
};

use ruse_ui_bridge::{UiEventPayload, UiJsonFrame};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use thiserror::Error;

use crate::{
    ApprovalPanelAction, ComposerAction, ComposerCursorMove, CrosstermTerminalBackend,
    LocalDiagnosticSeverity, ModelPickerAction, ReasoningEffort, SPINNER_FRAME_INTERVAL_MS,
    SessionPanelAction, StatusBarState, TerminalModeError, TerminalModeGuard, TerminalTheme,
    ThemePickerAction, TranscriptItem, TranscriptItemKind, TuiAction, TuiApp, TuiEffect, TuiScreen,
    render_app,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalAppConfig {
    pub status: StatusBarState,
    pub theme: TerminalTheme,
    pub tick_rate: Duration,
    pub startup_frames: Vec<UiJsonFrame>,
}

impl TerminalAppConfig {
    #[must_use]
    pub fn new(status: StatusBarState) -> Self {
        Self {
            status,
            theme: TerminalTheme::default(),
            tick_rate: Duration::from_millis(50),
            startup_frames: Vec::new(),
        }
    }
}

impl Default for TerminalAppConfig {
    fn default() -> Self {
        Self::new(StatusBarState::default())
    }
}

#[derive(Debug, Error)]
pub enum TerminalAppError {
    #[error("{0}")]
    TerminalMode(#[from] TerminalModeError),
    #[error("terminal app I/O failed: {0}")]
    Io(#[from] io::Error),
}

pub type TerminalFrameSender = Sender<UiJsonFrame>;

#[derive(Clone, Debug, Default)]
pub struct TerminalCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl TerminalCancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TerminalTurnOptions {
    pub reasoning_effort: ReasoningEffort,
}

pub trait TerminalTurnRuntime: Send + Sync + 'static {
    fn run_turn(
        &self,
        prompt: String,
        sender: TerminalFrameSender,
        cancellation: TerminalCancellationToken,
    );

    fn run_turn_with_options(
        &self,
        prompt: String,
        sender: TerminalFrameSender,
        cancellation: TerminalCancellationToken,
        _options: TerminalTurnOptions,
    ) {
        self.run_turn(prompt, sender, cancellation);
    }
}

pub trait TerminalCommandRuntime: Send + Sync + 'static {
    fn handle_effect(&self, effect: TuiEffect, sender: TerminalFrameSender);
}

pub struct TerminalEffectRuntime<'a> {
    pub turn_runtime: &'a Arc<dyn TerminalTurnRuntime>,
    pub command_runtime: &'a Arc<dyn TerminalCommandRuntime>,
    pub sender: &'a TerminalFrameSender,
    /// Single owner of the loop's turn-lifecycle state: whether a turn is active,
    /// its cancellation handle, and the prompts queued behind it. This replaces
    /// the three separate `&mut` params that used to be threaded through every
    /// lifecycle function, giving turn truth one home.
    pub turn: &'a mut TurnController,
}

/// Owns the event loop's turn-lifecycle state as a single unit: the active flag,
/// the active turn's cancellation handle, and prompts queued behind it. Folding
/// these three together (instead of threading three `&mut` params everywhere)
/// keeps turn truth in one place and collapses the signature bloat across
/// `start_prompt_turn` / `start_next_queued_prompt` / `drain_terminal_frames` and
/// the effect-runtime.
#[derive(Debug, Default)]
pub struct TurnController {
    /// Whether a turn is currently running. Mirrors `TuiApp::turn_active()`; kept
    /// here so the loop can gate queueing/draining without borrowing the app.
    pub active: bool,
    /// Cancellation handle for the in-flight turn, if any.
    pub cancellation: Option<TerminalCancellationToken>,
    /// Prompts submitted while a turn was active, drained in order once it ends.
    pub queued_prompts: VecDeque<TerminalQueuedPrompt>,
}

impl TurnController {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalQueuedPrompt {
    pub prompt: String,
    pub start_title: String,
    pub start_body: String,
    pub record_user_on_start: bool,
    pub options: TerminalTurnOptions,
}

impl TerminalQueuedPrompt {
    #[must_use]
    pub fn user(prompt: impl Into<String>, options: TerminalTurnOptions) -> Self {
        Self {
            prompt: prompt.into(),
            start_title: "queue".to_string(),
            start_body: "starting queued prompt".to_string(),
            record_user_on_start: false,
            options,
        }
    }

    #[must_use]
    pub fn recovery(
        prompt: impl Into<String>,
        summary: impl Into<String>,
        options: TerminalTurnOptions,
    ) -> Self {
        Self {
            prompt: prompt.into(),
            start_title: "recovery".to_string(),
            start_body: format!("starting recovered continuation: {}", summary.into()),
            record_user_on_start: false,
            options,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardCommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlatformClipboardResult {
    pub attempted: bool,
    pub command: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClipboardCopyReport {
    pub osc52_ok: bool,
    pub platform: PlatformClipboardResult,
}

#[derive(Debug, Default)]
pub struct DiagnosticTerminalTurnRuntime;

#[derive(Debug, Default)]
pub struct DiagnosticTerminalCommandRuntime;

impl TerminalTurnRuntime for DiagnosticTerminalTurnRuntime {
    fn run_turn(
        &self,
        _prompt: String,
        sender: TerminalFrameSender,
        _cancellation: TerminalCancellationToken,
    ) {
        let _ = sender.send(UiJsonFrame::Error {
            code: "terminal_runtime_unavailable".to_string(),
            message: "terminal streaming runtime is not configured".to_string(),
            recoverable: true,
        });
        let _ = sender.send(UiJsonFrame::Done { event_count: 0 });
    }
}

impl TerminalCommandRuntime for DiagnosticTerminalCommandRuntime {
    fn handle_effect(&self, effect: TuiEffect, sender: TerminalFrameSender) {
        let effect_name = match effect {
            TuiEffect::ResolveApproval { .. } => "resolve_approval",
            TuiEffect::ApprovalAborted(_) => "approval_abort",
            TuiEffect::CommandRetryPlan { .. } => "command_retry_plan",
            TuiEffect::ConversationCompact { .. } => "conversation_compact",
            TuiEffect::RecoveryStatus { .. } => "recovery_status",
            TuiEffect::RecoveryResumeDryRun { .. } => "recovery_resume_dry_run",
            TuiEffect::RecoveryResume { .. } => "recovery_resume",
            TuiEffect::SwitchSession { .. } => "switch_session",
            TuiEffect::SwitchModel { .. } => "switch_model",
            TuiEffect::NewSession => "new_session",
            TuiEffect::CopyToClipboard { .. } => "copy_to_clipboard",
            TuiEffect::PersistTerminalTheme { .. } => "persist_terminal_theme",
            TuiEffect::RunSlashCommand(_) => "slash_command",
            TuiEffect::None
            | TuiEffect::SubmitPrompt(_)
            | TuiEffect::SubmitRecoveryContinuation { .. }
            | TuiEffect::LocalDiagnostic { .. }
            | TuiEffect::ComposerBlurred => "local_effect",
        };
        let _ = sender.send(UiJsonFrame::Error {
            code: "terminal_command_runtime_unavailable".to_string(),
            message: format!("{effect_name} is not wired to the terminal command runtime"),
            recoverable: true,
        });
    }
}

pub fn run_terminal_app_stdio(config: TerminalAppConfig) -> Result<(), TerminalAppError> {
    run_terminal_app_stdio_with_runtime(config, Arc::new(DiagnosticTerminalTurnRuntime))
}

pub fn run_terminal_app_stdio_with_runtime(
    config: TerminalAppConfig,
    turn_runtime: Arc<dyn TerminalTurnRuntime>,
) -> Result<(), TerminalAppError> {
    run_terminal_app_stdio_with_runtimes(
        config,
        turn_runtime,
        Arc::new(DiagnosticTerminalCommandRuntime),
    )
}

pub fn run_terminal_app_stdio_with_runtimes(
    config: TerminalAppConfig,
    turn_runtime: Arc<dyn TerminalTurnRuntime>,
    command_runtime: Arc<dyn TerminalCommandRuntime>,
) -> Result<(), TerminalAppError> {
    let TerminalAppConfig {
        status,
        theme,
        tick_rate,
        startup_frames,
    } = config;
    // Mouse capture is on so left-clicks can expand/collapse a specific thinking
    // or tool block ("see which, click which"). Trade-off: native drag-select is
    // disabled while capture is active — use /copy or Ctrl+Shift+C for clipboard.
    // Bracketed paste still keeps multi-line pastes atomic.
    let mut guard = TerminalModeGuard::enter(CrosstermTerminalBackend)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let size = terminal.size()?;
    let mut app = build_terminal_app_state(status, theme, size.width, size.height, startup_frames);

    let (frame_sender, frame_receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    // Idle poll timeout. When no turn is active we block for this long waiting on
    // input; the runtime frame channel is not integrated into crossterm's poll,
    // so we cannot block indefinitely (a `Done` frame arriving while idle must
    // still be drained). We reuse the configured `tick_rate` but floor it well
    // above the previous fixed 50ms so idle wakeups drop sharply (from ~20/sec)
    // while a queued/late frame is still drained promptly.
    let idle_poll = tick_rate.max(Duration::from_millis(250));

    // Draw once up front, then only when state changes (dirty-flag gating): idle
    // CPU drops to near zero instead of repainting 20x/second forever.
    let mut dirty = true;

    while !app.should_quit {
        if drain_terminal_frames(&mut app, &frame_receiver, &mut turn) > 0 {
            dirty = true;
        }
        start_next_queued_prompt(&mut app, &turn_runtime, &frame_sender, &mut turn);

        // Advance the spinner from wall-clock time so its cadence is independent
        // of poll frequency and it never freezes while the user is typing.
        if app.refresh_spinner(Instant::now()) {
            dirty = true;
        }
        // Expire a transient toast once its TTL elapses.
        if app.expire_toast(Instant::now()) {
            dirty = true;
        }

        if dirty {
            terminal.draw(|frame| render_app(frame, &app))?;
            dirty = false;
        }

        // Poll at frame cadence while a turn animates; block longer when idle. A
        // visible toast also needs frame-cadence polling so it expires on time.
        let poll_timeout = if app.turn_active() || app.toast.is_some() {
            Duration::from_millis(SPINNER_FRAME_INTERVAL_MS as u64)
        } else {
            idle_poll
        };

        if event::poll(poll_timeout)? {
            // Any consumed input event mutates or may mutate visible state.
            dirty = true;
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = key_event_to_action_for_app(key, &app) {
                        if action == TuiAction::CancelTurnRequested {
                            handle_cancel_turn_requested(
                                &mut app,
                                turn.active,
                                turn.cancellation.as_ref(),
                            );
                            continue;
                        }
                        let effect = app.reduce(action);
                        apply_terminal_effect_with_runtime_and_queue(
                            &mut app,
                            effect,
                            TerminalEffectRuntime {
                                turn_runtime: &turn_runtime,
                                command_runtime: &command_runtime,
                                sender: &frame_sender,
                                turn: &mut turn,
                            },
                        );
                    }
                }
                Event::Resize(width, height) => {
                    app.reduce(TuiAction::Resize { width, height });
                }
                Event::Paste(text) => {
                    let effect = app.reduce(TuiAction::Composer(ComposerAction::InsertText(text)));
                    apply_terminal_effect_with_runtime_and_queue(
                        &mut app,
                        effect,
                        TerminalEffectRuntime {
                            turn_runtime: &turn_runtime,
                            command_runtime: &command_runtime,
                            sender: &frame_sender,
                            turn: &mut turn,
                        },
                    );
                }
                Event::Mouse(mouse) => {
                    if let Some(action) = mouse_event_to_action(mouse) {
                        let effect = app.reduce(action);
                        apply_terminal_effect_with_runtime_and_queue(
                            &mut app,
                            effect,
                            TerminalEffectRuntime {
                                turn_runtime: &turn_runtime,
                                command_runtime: &command_runtime,
                                sender: &frame_sender,
                                turn: &mut turn,
                            },
                        );
                    }
                }
                Event::FocusGained | Event::FocusLost => {}
            }
        }

        // Advance the spinner from wall-clock time (independent of whether an
        // input event or a poll timeout woke us). Mark dirty only when the frame
        // index actually changed so an idle app still parks at ~0% CPU.
        if app.refresh_spinner(Instant::now()) {
            dirty = true;
        }
    }

    terminal.show_cursor()?;
    guard.restore()?;
    Ok(())
}

#[must_use]
pub fn build_terminal_app_state(
    status: StatusBarState,
    theme: TerminalTheme,
    width: u16,
    height: u16,
    startup_frames: Vec<UiJsonFrame>,
) -> TuiApp {
    let mut app = TuiApp::new(status);
    app.theme = theme;
    app.refresh_status_panel("none");
    app.reduce(TuiAction::Resize { width, height });
    for frame in startup_frames {
        app.reduce(TuiAction::BridgeFrame(Box::new(frame)));
    }
    app
}

#[must_use]
pub fn mouse_event_to_action(mouse: MouseEvent) -> Option<TuiAction> {
    // Mouse capture is on so we can:
    //   - left-click expand/collapse a specific thinking/tool block
    //   - drag-select transcript text and copy on release
    // Native host selection is disabled while capture is active; in-app
    // selection restores the select+copy path.
    match mouse.kind {
        MouseEventKind::ScrollUp => Some(TuiAction::ScrollTranscriptUp),
        MouseEventKind::ScrollDown => Some(TuiAction::ScrollTranscriptDown),
        MouseEventKind::Down(MouseButton::Left) => {
            // Shift+click: leave to host terminal (if it still synthesizes
            // selection); we do not start an in-app drag.
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                return None;
            }
            Some(TuiAction::SelectionDragStart {
                column: mouse.column,
                row: mouse.row,
            })
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                return None;
            }
            Some(TuiAction::SelectionDragUpdate {
                column: mouse.column,
                row: mouse.row,
            })
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                return None;
            }
            Some(TuiAction::SelectionDragEnd {
                column: mouse.column,
                row: mouse.row,
            })
        }
        _ => None,
    }
}

#[must_use]
pub fn key_event_to_action(key: KeyEvent) -> Option<TuiAction> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => Some(TuiAction::CancelTurnRequested),
            KeyCode::Char('e') | KeyCode::Char('E') => Some(TuiAction::TogglePanelExpansion),
            KeyCode::Char('l') | KeyCode::Char('L') => Some(TuiAction::ClearTranscript),
            KeyCode::Char('h') | KeyCode::Char('H') => Some(TuiAction::OpenHelp),
            // Expand/collapse the latest tool result (shell output, long observations).
            // Clipboard copy stays on Ctrl+Shift+O / the /copy slash command so
            // Ctrl+O matches the on-screen "ctrl+o to expand" hint.
            KeyCode::Char('o') | KeyCode::Char('O') => Some(TuiAction::ToggleToolOutput),
            KeyCode::Char('r') | KeyCode::Char('R') => Some(TuiAction::StartRecoveryContinuation),
            // Open the theme picker (also available via `/theme`). Up/Down live-previews;
            // Enter confirms; Esc restores the previous theme.
            KeyCode::Char('t') | KeyCode::Char('T') => Some(TuiAction::CycleTheme),
            // Expand/collapse the reasoning ("thinking") block.
            KeyCode::Char('p') | KeyCode::Char('P') => Some(TuiAction::ToggleReasoning),
            // Open the model-switch picker (also available via `/model`). Ctrl+M is
            // avoided because most terminals send it as Enter.
            KeyCode::Char('g') | KeyCode::Char('G') => {
                Some(TuiAction::ModelPicker(ModelPickerAction::Open))
            }
            // Open the session picker (also available via `/sessions`).
            KeyCode::Char('s') | KeyCode::Char('S') => {
                Some(TuiAction::SessionPanel(SessionPanelAction::Open))
            }
            // Ctrl+Up/Down also navigate composer history (alongside Alt+Up/Down).
            KeyCode::Up => Some(TuiAction::Composer(ComposerAction::HistoryPrevious)),
            KeyCode::Down => Some(TuiAction::Composer(ComposerAction::HistoryNext)),
            _ => None,
        };
    }

    // Alt+Up / Alt+Down: explicit composer history. Bare Up/Down scroll the
    // transcript so a mouse wheel (synthesized as arrow keys without mouse
    // capture) never cycles prompt history.
    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            KeyCode::Up => Some(TuiAction::Composer(ComposerAction::HistoryPrevious)),
            KeyCode::Down => Some(TuiAction::Composer(ComposerAction::HistoryNext)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char(character) => {
            Some(TuiAction::Composer(ComposerAction::InsertChar(character)))
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(TuiAction::Composer(ComposerAction::InsertNewline))
        }
        KeyCode::Enter => Some(TuiAction::Composer(ComposerAction::Submit)),
        KeyCode::Backspace => Some(TuiAction::Composer(ComposerAction::Backspace)),
        // Bare Up/Down scroll the transcript. Multiline caret motion is handled
        // in `key_event_to_action_for_app` when the caret is mid-buffer.
        KeyCode::Up => Some(TuiAction::ScrollTranscriptUp),
        KeyCode::Down => Some(TuiAction::ScrollTranscriptDown),
        KeyCode::Left => Some(TuiAction::Composer(ComposerAction::MoveCursor(
            ComposerCursorMove::Left,
        ))),
        KeyCode::Right => Some(TuiAction::Composer(ComposerAction::MoveCursor(
            ComposerCursorMove::Right,
        ))),
        KeyCode::PageDown => Some(TuiAction::ScrollTranscriptPageDown),
        KeyCode::PageUp => Some(TuiAction::ScrollTranscriptPageUp),
        KeyCode::Home => Some(TuiAction::ScrollTranscriptTop),
        KeyCode::End => Some(TuiAction::ScrollTranscriptBottom),
        KeyCode::Tab => Some(TuiAction::CyclePermissionMode),
        KeyCode::Esc => Some(TuiAction::OpenChat),
        KeyCode::F(1) => Some(TuiAction::OpenHelp),
        _ => None,
    }
}

#[must_use]
pub fn key_event_to_action_for_app(key: KeyEvent, app: &TuiApp) -> Option<TuiAction> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }

    // Reject-reason capture owns the keyboard while active: the user is typing a
    // one-line reason that becomes the deny/steering message. This must win over
    // every other route (including Ctrl+C, so a stray chord doesn't escape it).
    if app.reject_reason.is_some() {
        return match key.code {
            KeyCode::Esc => Some(TuiAction::ApprovalRejectCancel),
            KeyCode::Enter => Some(TuiAction::ApprovalRejectSubmit),
            KeyCode::Backspace => Some(TuiAction::ApprovalRejectBackspace),
            KeyCode::Char(character) => Some(TuiAction::ApprovalRejectInput(character)),
            _ => None,
        };
    }

    // The approval modal is a true modal: while it is visible it captures all key
    // routing (skipping the composer keymap entirely) so decision keys never leak
    // into the composer. This is checked BEFORE Ctrl+C and the composer.
    if app.approval_modal_visible() {
        return match key.code {
            KeyCode::Char('1') => Some(TuiAction::ApprovalPanel(ApprovalPanelAction::Allow)),
            KeyCode::Char('2') => Some(TuiAction::ApprovalPanel(ApprovalPanelAction::AllowAlways)),
            KeyCode::Char('3') => Some(TuiAction::ApprovalRejectBegin),
            // Up/Down move the highlighted option; Enter confirms it. The default
            // highlight is Allow once.
            KeyCode::Up => Some(TuiAction::ApprovalSelectPrevious),
            KeyCode::Down => Some(TuiAction::ApprovalSelectNext),
            KeyCode::Enter => Some(TuiAction::ApprovalConfirmSelection),
            // Tab/BackTab switch between multiple pending requests in the queue.
            KeyCode::Tab => Some(TuiAction::ApprovalPanel(ApprovalPanelAction::FocusNext)),
            KeyCode::BackTab => Some(TuiAction::ApprovalPanel(ApprovalPanelAction::FocusPrevious)),
            // Esc is a non-destructive dismiss: it hides the modal but leaves the
            // request pending (the footer keeps showing the pending indicator).
            // It never denies.
            KeyCode::Esc => Some(TuiAction::ApprovalModalDismiss),
            _ => None,
        };
    }

    // The model picker is a true modal, same shape as the approval modal: while
    // open it captures all key routing so Up/Down/Enter/Esc drive the selection
    // and never leak into the composer. Checked before Ctrl+C and the composer.
    if app.model_picker.is_open() {
        return match key.code {
            KeyCode::Up => Some(TuiAction::ModelPicker(ModelPickerAction::FocusPrevious)),
            KeyCode::Down => Some(TuiAction::ModelPicker(ModelPickerAction::FocusNext)),
            KeyCode::Enter => Some(TuiAction::ModelPicker(ModelPickerAction::Confirm)),
            KeyCode::Esc => Some(TuiAction::ModelPicker(ModelPickerAction::Dismiss)),
            _ => None,
        };
    }

    // The session picker is the same centered-modal shape. `n` starts a fresh
    // session; Enter switches to the highlighted one; Esc dismisses. While open it
    // owns the keyboard so the keys never leak into the composer.
    if app.session_panel.is_open() {
        return match key.code {
            KeyCode::Up => Some(TuiAction::SessionPanel(SessionPanelAction::FocusPrevious)),
            KeyCode::Down => Some(TuiAction::SessionPanel(SessionPanelAction::FocusNext)),
            KeyCode::Enter => Some(TuiAction::SessionPanel(SessionPanelAction::SwitchFocused)),
            KeyCode::Char('n') | KeyCode::Char('N') => {
                Some(TuiAction::SessionPanel(SessionPanelAction::NewSession))
            }
            KeyCode::Esc => Some(TuiAction::SessionPanel(SessionPanelAction::Dismiss)),
            _ => None,
        };
    }

    // Theme picker: Up/Down live-preview the focused theme; Enter persists it;
    // Esc restores the theme that was active when the picker opened.
    if app.theme_picker.is_open() {
        return match key.code {
            KeyCode::Up => Some(TuiAction::ThemePicker(ThemePickerAction::FocusPrevious)),
            KeyCode::Down => Some(TuiAction::ThemePicker(ThemePickerAction::FocusNext)),
            KeyCode::Enter => Some(TuiAction::ThemePicker(ThemePickerAction::Confirm)),
            KeyCode::Esc => Some(TuiAction::ThemePicker(ThemePickerAction::Dismiss)),
            _ => None,
        };
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return key_event_to_action(key);
    }

    // Shift+Tab changes the reasoning effort for subsequent turns. Modal-local
    // BackTab behavior is handled above before this global chat shortcut.
    if key.code == KeyCode::BackTab && app.screen == TuiScreen::Chat {
        return Some(TuiAction::CycleReasoningEffort);
    }

    // Esc is the primary interrupt while a turn is active (matching Claude Code /
    // Codex); Ctrl+C remains a fallback (and the force-quit escalation).
    if key.code == KeyCode::Esc && app.turn_active() {
        return Some(TuiAction::CancelTurnRequested);
    }

    if app.composer.slash_suggestions.visible() {
        return match key.code {
            KeyCode::Up => Some(TuiAction::Composer(ComposerAction::SlashSuggestionPrevious)),
            KeyCode::Down => Some(TuiAction::Composer(ComposerAction::SlashSuggestionNext)),
            KeyCode::Tab => Some(TuiAction::Composer(ComposerAction::SlashSuggestionComplete)),
            KeyCode::Esc => Some(TuiAction::Composer(ComposerAction::Escape)),
            KeyCode::Enter if slash_suggestion_should_complete_on_enter(app) => {
                // Enter on a slash-menu selection completes AND runs in one step;
                // Tab (above) completes without running.
                Some(TuiAction::Composer(
                    ComposerAction::SlashSuggestionCompleteAndRun,
                ))
            }
            _ => key_event_to_action(key),
        };
    }

    if app.diff_panel.is_some() {
        return match key.code {
            KeyCode::PageDown => Some(TuiAction::DiffPanel(crate::DiffPanelAction::ScrollDown)),
            KeyCode::PageUp => Some(TuiAction::DiffPanel(crate::DiffPanelAction::ScrollUp)),
            _ => key_event_to_action(key),
        };
    }

    // In a multi-line composer draft, bare Up/Down move the caret within the
    // buffer until they hit the first/last line — only then do they fall through
    // to transcript scroll (or Alt/Ctrl+Up/Down for history).
    if !key.modifiers.contains(KeyModifiers::ALT)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && app.composer.line_count() > 1
    {
        let (row, _) = app.composer.cursor();
        match key.code {
            KeyCode::Up if row > 0 => {
                return Some(TuiAction::Composer(ComposerAction::MoveCursor(
                    ComposerCursorMove::Up,
                )));
            }
            KeyCode::Down if row + 1 < app.composer.line_count() => {
                return Some(TuiAction::Composer(ComposerAction::MoveCursor(
                    ComposerCursorMove::Down,
                )));
            }
            _ => {}
        }
    }

    key_event_to_action(key)
}

fn slash_suggestion_should_complete_on_enter(app: &TuiApp) -> bool {
    if crate::SlashCommand::parse(&app.composer.text()).is_ok() {
        return false;
    }

    app.composer.slash_suggestions.selected_command().is_some()
}

pub fn apply_terminal_effect(app: &mut TuiApp, effect: TuiEffect) {
    let (sender, receiver) = mpsc::channel();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let command_runtime: Arc<dyn TerminalCommandRuntime> =
        Arc::new(DiagnosticTerminalCommandRuntime);
    let mut turn = TurnController::new();
    apply_terminal_effect_with_runtimes(
        app,
        effect,
        &runtime,
        &command_runtime,
        &sender,
        &mut turn,
    );
    drain_terminal_frames(app, &receiver, &mut turn);
}

pub fn apply_terminal_effect_with_runtime(
    app: &mut TuiApp,
    effect: TuiEffect,
    runtime: &Arc<dyn TerminalTurnRuntime>,
    sender: &TerminalFrameSender,
    turn: &mut TurnController,
) {
    let command_runtime: Arc<dyn TerminalCommandRuntime> =
        Arc::new(DiagnosticTerminalCommandRuntime);
    apply_terminal_effect_with_runtimes(app, effect, runtime, &command_runtime, sender, turn);
}

pub fn apply_terminal_effect_with_runtimes(
    app: &mut TuiApp,
    effect: TuiEffect,
    runtime: &Arc<dyn TerminalTurnRuntime>,
    command_runtime: &Arc<dyn TerminalCommandRuntime>,
    sender: &TerminalFrameSender,
    turn: &mut TurnController,
) {
    apply_terminal_effect_with_runtime_and_queue(
        app,
        effect,
        TerminalEffectRuntime {
            turn_runtime: runtime,
            command_runtime,
            sender,
            turn,
        },
    );
}

pub fn apply_terminal_effect_with_runtime_and_queue(
    app: &mut TuiApp,
    effect: TuiEffect,
    runtime: TerminalEffectRuntime<'_>,
) {
    match effect {
        TuiEffect::None | TuiEffect::ComposerBlurred => {}
        TuiEffect::SubmitPrompt(prompt) => {
            if runtime.turn.active {
                app.transcript.push(TranscriptItem::new(
                    TranscriptItemKind::User,
                    "user queued",
                    prompt.clone(),
                    false,
                ));
                runtime
                    .turn
                    .queued_prompts
                    .push_back(TerminalQueuedPrompt::user(
                        prompt,
                        TerminalTurnOptions {
                            reasoning_effort: app.status.reasoning_effort,
                        },
                    ));
                return;
            }

            start_prompt_turn(
                app,
                prompt,
                runtime.turn_runtime,
                runtime.sender,
                runtime.turn,
                true,
                TerminalTurnOptions {
                    reasoning_effort: app.status.reasoning_effort,
                },
            );
        }
        TuiEffect::SubmitRecoveryContinuation { prompt, summary } => {
            if runtime.turn.active {
                app.transcript.push(TranscriptItem::new(
                    TranscriptItemKind::Progress,
                    "recovery queued",
                    summary.clone(),
                    false,
                ));
                runtime
                    .turn
                    .queued_prompts
                    .push_back(TerminalQueuedPrompt::recovery(
                        prompt,
                        summary,
                        TerminalTurnOptions {
                            reasoning_effort: app.status.reasoning_effort,
                        },
                    ));
                return;
            }

            app.transcript.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "recovery",
                format!("starting recovered continuation: {summary}"),
                false,
            ));
            start_prompt_turn(
                app,
                prompt,
                runtime.turn_runtime,
                runtime.sender,
                runtime.turn,
                false,
                TerminalTurnOptions {
                    reasoning_effort: app.status.reasoning_effort,
                },
            );
        }
        TuiEffect::LocalDiagnostic { message, severity } => {
            // Severity decides the destination: real errors render red in the
            // transcript; notes and warnings surface as a transient, neutral toast
            // that auto-expires (never a red chip).
            match severity {
                LocalDiagnosticSeverity::Error => {
                    app.transcript.push(TranscriptItem::new(
                        TranscriptItemKind::Diagnostic,
                        "diagnostic",
                        message,
                        false,
                    ));
                }
                LocalDiagnosticSeverity::Note | LocalDiagnosticSeverity::Warning => {
                    app.show_toast(message, severity, Instant::now());
                }
            }
        }
        TuiEffect::CopyToClipboard { text } => {
            // Clipboard copy is a success confirmation, not an error: it belongs in
            // the transient toast, never as a red transcript diagnostic.
            let diagnostic = copy_text_to_terminal_clipboard(&mut io::stdout(), &text).diagnostic();
            app.show_toast(diagnostic, LocalDiagnosticSeverity::Note, Instant::now());
        }
        effect @ TuiEffect::ResolveApproval { .. } => {
            // Approval continuation can include tool execution and another model
            // stream. Run it off the UI thread so the modal disappears and the TUI
            // keeps repainting while continuation frames arrive.
            let command_runtime = Arc::clone(runtime.command_runtime);
            let sender = runtime.sender.clone();
            thread::spawn(move || command_runtime.handle_effect(effect, sender));
        }
        effect @ (TuiEffect::SwitchSession { .. } | TuiEffect::NewSession) => {
            if let Some(cancellation) = runtime.turn.cancellation.as_ref() {
                cancellation.cancel();
            }
            runtime.turn.active = false;
            runtime.turn.cancellation = None;
            runtime.turn.queued_prompts.clear();
            app.mark_turn_finished();
            runtime
                .command_runtime
                .handle_effect(effect, runtime.sender.clone());
        }
        TuiEffect::RunSlashCommand(_)
        | TuiEffect::ApprovalAborted(_)
        | TuiEffect::CommandRetryPlan { .. }
        | TuiEffect::ConversationCompact { .. }
        | TuiEffect::PersistTerminalTheme { .. }
        | TuiEffect::RecoveryStatus { .. }
        | TuiEffect::RecoveryResumeDryRun { .. }
        | TuiEffect::RecoveryResume { .. }
        | TuiEffect::SwitchModel { .. } => {
            runtime
                .command_runtime
                .handle_effect(effect, runtime.sender.clone());
        }
    }
}

pub fn start_next_queued_prompt(
    app: &mut TuiApp,
    runtime: &Arc<dyn TerminalTurnRuntime>,
    sender: &TerminalFrameSender,
    turn: &mut TurnController,
) {
    if turn.active {
        return;
    }
    let Some(queued) = turn.queued_prompts.pop_front() else {
        return;
    };
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Progress,
        queued.start_title,
        queued.start_body,
        false,
    ));
    start_prompt_turn(
        app,
        queued.prompt,
        runtime,
        sender,
        turn,
        queued.record_user_on_start,
        queued.options,
    );
}

fn start_prompt_turn(
    app: &mut TuiApp,
    prompt: String,
    runtime: &Arc<dyn TerminalTurnRuntime>,
    sender: &TerminalFrameSender,
    turn: &mut TurnController,
    record_user: bool,
    options: TerminalTurnOptions,
) {
    if record_user {
        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::User,
            "user",
            prompt.clone(),
            false,
        ));
        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::Reasoning,
            "thinking",
            "thinking",
            true,
        ));
    }
    turn.active = true;
    // Anchor the authoritative turn clock. This drives the spinner and the
    // elapsed-time readout regardless of whether a user/thinking placeholder was
    // recorded, so queued and recovery-continuation turns show the live
    // indicator immediately instead of only once the model starts streaming.
    app.mark_turn_started(std::time::Instant::now());
    let cancellation = TerminalCancellationToken::new();
    turn.cancellation = Some(cancellation.clone());
    let sender = sender.clone();
    let runtime = Arc::clone(runtime);
    thread::spawn(move || {
        runtime.run_turn_with_options(prompt, sender, cancellation, options);
    });
}

pub fn drain_terminal_frames(
    app: &mut TuiApp,
    receiver: &Receiver<UiJsonFrame>,
    turn: &mut TurnController,
) -> usize {
    let mut drained = 0;
    while let Ok(frame) = receiver.try_recv() {
        let is_done = matches!(frame, UiJsonFrame::Done { .. });
        let replaying_conversation = app.conversation_replay_pending;

        // Once a turn has been cancelled we treat its `Done` frame as the turn
        // boundary and drop every other frame it emits in the meantime. A
        // detached turn thread keeps running until it observes the cancellation
        // token cooperatively; without this guard its late deltas would keep
        // streaming into the transcript, so cancel would not feel instant. The
        // `Done` frame is the wire-level end-of-turn marker, so gating on it
        // discards exactly the superseded turn's tail without needing a
        // per-frame generation id (the shared channel is single-turn: the
        // `turn_active` gate blocks the next turn from starting until this
        // `Done` drains, so frames from two turns can never interleave here).
        let cancelled = app.cancel_requested;

        if is_done {
            turn.active = false;
            turn.cancellation = None;
            // Clear the authoritative turn clock so the spinner/elapsed readout
            // stops and the loop can drop back to blocking (idle) redraw. This
            // also clears `cancel_requested`, ending the drop-stale-frames window.
            app.mark_turn_finished();
            app.conversation_replay_pending = false;
        }

        // Live submission already inserts the user message optimistically, so its
        // runtime echo is dropped. Session replay has no optimistic insertion and
        // must retain persisted user messages until its Done boundary arrives.
        if is_runtime_echoed_user_message(&frame) && !replaying_conversation {
            continue;
        }

        // Drop the cancelled turn's tail frames (everything but its `Done`, which
        // was already consumed above to end the turn).
        if cancelled && !is_done {
            drained += 1;
            continue;
        }

        app.reduce(TuiAction::BridgeFrame(Box::new(frame)));
        drained += 1;
    }
    drained
}

pub fn handle_cancel_turn_requested(
    app: &mut TuiApp,
    turn_active: bool,
    cancellation: Option<&TerminalCancellationToken>,
) {
    if !turn_active {
        // Nothing running: Ctrl+C quits, matching terminal convention.
        app.reduce(TuiAction::QuitRequested);
        return;
    }

    if app.cancel_requested {
        // Second Ctrl+C while a turn is still running (cooperative cancel did not
        // land — a wedged tool or unresponsive provider). Escalate to a hard quit
        // so the user is never trapped; the terminal guard still restores cleanly.
        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::Progress,
            "turn",
            "force quit (turn did not stop)",
            false,
        ));
        app.reduce(TuiAction::QuitRequested);
        return;
    }

    // First Ctrl+C: request cooperative cancellation once, and record it once.
    if let Some(cancellation) = cancellation {
        cancellation.cancel();
    }
    app.cancel_requested = true;
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Progress,
        "turn",
        "cancellation requested (Ctrl+C again to force quit)",
        false,
    ));
}

fn is_runtime_echoed_user_message(frame: &UiJsonFrame) -> bool {
    matches!(
        frame,
        UiJsonFrame::Event(event)
            if matches!(event.payload, UiEventPayload::UserMessage { .. })
    )
}

#[must_use]
pub fn osc52_copy_sequence(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", STANDARD.encode(text.as_bytes()))
}

pub fn write_osc52_clipboard(writer: &mut impl Write, text: &str) -> io::Result<()> {
    writer.write_all(osc52_copy_sequence(text).as_bytes())?;
    writer.flush()
}

pub fn copy_text_to_terminal_clipboard(writer: &mut impl Write, text: &str) -> ClipboardCopyReport {
    copy_text_to_terminal_clipboard_with_platform_result(
        writer,
        text,
        copy_text_to_platform_clipboard(text),
    )
}

pub fn copy_text_to_terminal_clipboard_with_platform_result(
    writer: &mut impl Write,
    text: &str,
    platform: PlatformClipboardResult,
) -> ClipboardCopyReport {
    ClipboardCopyReport {
        osc52_ok: write_osc52_clipboard(writer, text).is_ok(),
        platform,
    }
}

impl ClipboardCopyReport {
    #[must_use]
    pub fn copied(&self) -> bool {
        self.osc52_ok || self.platform.command.is_some()
    }

    #[must_use]
    pub fn diagnostic(&self) -> String {
        match (self.osc52_ok, self.platform.command.as_deref()) {
            (true, Some(command)) => {
                format!("latest assistant output copied via OSC52 and {command}")
            }
            (true, None) => {
                "latest assistant output copied via OSC52 terminal clipboard".to_string()
            }
            (false, Some(command)) => {
                format!("latest assistant output copied via {command}")
            }
            (false, None) if self.platform.attempted => format!(
                "terminal clipboard copy failed: {}",
                self.platform
                    .error
                    .as_deref()
                    .unwrap_or("no platform clipboard command succeeded")
            ),
            (false, None) => "terminal clipboard copy failed: OSC52 write failed and no platform clipboard command is available".to_string(),
        }
    }
}

pub fn copy_text_to_platform_clipboard(text: &str) -> PlatformClipboardResult {
    let candidates = platform_clipboard_command_candidates();
    if candidates.is_empty() {
        return PlatformClipboardResult::default();
    }

    let mut last_error = None;
    for spec in candidates {
        match run_platform_clipboard_command(&spec, text) {
            Ok(()) => {
                return PlatformClipboardResult {
                    attempted: true,
                    command: Some(spec.display()),
                    error: None,
                };
            }
            Err(error) => {
                last_error = Some(format!("{}: {error}", spec.display()));
            }
        }
    }

    PlatformClipboardResult {
        attempted: true,
        command: None,
        error: last_error,
    }
}

#[must_use]
pub fn platform_clipboard_command_candidates() -> Vec<ClipboardCommandSpec> {
    platform_clipboard_command_candidates_for(
        env::consts::OS,
        env::var("WAYLAND_DISPLAY").ok().as_deref(),
        env::var("DISPLAY").ok().as_deref(),
    )
}

#[must_use]
pub fn platform_clipboard_command_candidates_for(
    os: &str,
    wayland_display: Option<&str>,
    display: Option<&str>,
) -> Vec<ClipboardCommandSpec> {
    match os {
        "macos" => vec![ClipboardCommandSpec::new("pbcopy", [])],
        "windows" => vec![ClipboardCommandSpec::new(
            "powershell",
            ["-NoProfile", "-Command", "Set-Clipboard"],
        )],
        "linux" | "freebsd" | "openbsd" | "netbsd" => {
            let mut candidates = Vec::new();
            if wayland_display.is_some_and(|value| !value.trim().is_empty()) {
                candidates.push(ClipboardCommandSpec::new("wl-copy", []));
            }
            if display.is_some_and(|value| !value.trim().is_empty()) {
                candidates.push(ClipboardCommandSpec::new(
                    "xclip",
                    ["-selection", "clipboard"],
                ));
                candidates.push(ClipboardCommandSpec::new(
                    "xsel",
                    ["--clipboard", "--input"],
                ));
            }
            candidates
        }
        _ => Vec::new(),
    }
}

impl ClipboardCommandSpec {
    #[must_use]
    pub fn new<const N: usize>(program: impl Into<String>, args: [&str; N]) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(ToString::to_string).collect(),
        }
    }

    #[must_use]
    pub fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

fn run_platform_clipboard_command(spec: &ClipboardCommandSpec, text: &str) -> io::Result<()> {
    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "clipboard command exited with {status}"
        )))
    }
}
