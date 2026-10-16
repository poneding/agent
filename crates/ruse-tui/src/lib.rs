#![forbid(unsafe_code)]
//! Terminal UI shell contracts for agent.

pub mod app;
pub mod approval_panel;
pub mod command_panel;
pub mod composer;
pub mod diff_panel;
pub mod diff_view;
pub mod extension_panel;
pub mod hit_test;
pub mod markdown_view;
pub mod model_picker;
pub mod palette;
pub mod render;
pub mod selection;
pub mod session_panel;
pub mod shell_highlight;
pub mod slash;
pub mod status_panel;
pub mod task_panel;
pub mod terminal;
pub mod terminal_app;
pub mod theme_picker;
pub mod trace_panel;
pub mod transcript;
pub mod welcome;

pub use app::{
    AllowRule, ApprovalOption, CommandRetryCandidateState, ConnectionState, DiffPanelSource,
    PermissionMode, ReasoningEffort, RecoveryContinuationState, RejectReasonInput,
    SPINNER_FRAME_COUNT, SPINNER_FRAME_INTERVAL_MS, StatusBarState, TOAST_TTL, TerminalSize,
    TerminalTheme, TuiAction, TuiApp, TuiScreen, TuiSidePanel, TuiToast, TurnState,
};
pub use approval_panel::{ApprovalPanelAction, ApprovalPanelState};
pub use command_panel::{CommandPanelAction, CommandPanelOutput, CommandPanelState};
pub use composer::{
    COMPOSER_PLACEHOLDER, ComposerAction, ComposerCursorMove, ComposerState,
    LocalDiagnosticSeverity, TuiBridgeContext, TuiEffect,
};
pub use diff_panel::{DiffPanelAction, DiffPanelState};
pub use extension_panel::ExtensionPanelState;
pub use hit_test::{TranscriptHit, hit_test_transcript, hit_test_transcript_for_test};
pub use model_picker::{ModelPickerAction, ModelPickerEntry, ModelPickerState};
pub use palette::{Palette, Semantic};
pub use render::{
    paint_transcript_history_lines, render_app, render_app_to_string, render_sticky_chrome,
    sticky_chrome_height, transcript_history_lines,
};
pub use selection::{
    ContentGeometry, SelectionRange, SelectionRegion, SelectionState, extract_selected_text,
    extract_selection,
};
pub use session_panel::{SessionPanelAction, SessionPanelState};
pub use slash::{
    SlashCommand, SlashCommandError, SlashSuggestionItem, SlashSuggestionState,
    ThinkingVisibilityCommand,
};
pub use status_panel::StatusPanelState;
pub use task_panel::{TaskPanelNotification, TaskPanelState};
pub use terminal::{
    CrosstermTerminalBackend, FixtureTerminalBackend, TerminalModeBackend, TerminalModeError,
    TerminalModeGuard,
};
pub use terminal_app::{
    ClipboardCommandSpec, ClipboardCopyReport, DiagnosticTerminalCommandRuntime,
    DiagnosticTerminalTurnRuntime, PlatformClipboardResult, TerminalAppConfig, TerminalAppError,
    TerminalCancellationToken, TerminalCommandRuntime, TerminalEffectRuntime, TerminalFrameSender,
    TerminalQueuedPrompt, TerminalTurnOptions, TerminalTurnRuntime, TurnController,
    apply_terminal_effect, apply_terminal_effect_with_runtime,
    apply_terminal_effect_with_runtime_and_queue, apply_terminal_effect_with_runtimes,
    build_terminal_app_state, copy_text_to_terminal_clipboard,
    copy_text_to_terminal_clipboard_with_platform_result, drain_terminal_frames,
    handle_cancel_turn_requested, key_event_to_action, key_event_to_action_for_app,
    mouse_event_to_action, osc52_copy_sequence, platform_clipboard_command_candidates_for,
    run_terminal_app_stdio, run_terminal_app_stdio_with_runtime,
    run_terminal_app_stdio_with_runtimes, start_next_queued_prompt, write_osc52_clipboard,
};
pub use theme_picker::{ThemePickerAction, ThemePickerState};
pub use trace_panel::{TraceEntryKind, TraceFilter, TracePanelEntry, TracePanelState};
pub use transcript::{
    ReasoningVisibility, TranscriptItem, TranscriptItemKind, TranscriptState, WaitingActivity,
    estimate_thinking_tokens, friendly_tool_name,
};

#[must_use]
pub const fn launch_message() -> &'static str {
    "agent tui shell ready\n"
}
