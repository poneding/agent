#![forbid(unsafe_code)]
//! Terminal UI shell contracts for agent.

pub mod app;
pub mod approval_panel;
pub mod command_panel;
pub mod composer;
pub mod diff_panel;
pub mod extension_panel;
pub mod render;
pub mod session_panel;
pub mod slash;
pub mod status_panel;
pub mod task_panel;
pub mod terminal;
pub mod trace_panel;
pub mod transcript;

pub use app::{StatusBarState, TerminalSize, TuiAction, TuiApp, TuiScreen, TuiSidePanel};
pub use approval_panel::{ApprovalPanelAction, ApprovalPanelState};
pub use command_panel::CommandPanelState;
pub use composer::{ComposerAction, ComposerState, TuiBridgeContext, TuiEffect};
pub use diff_panel::DiffPanelState;
pub use extension_panel::ExtensionPanelState;
pub use render::{render_app, render_app_to_string};
pub use session_panel::{SessionPanelAction, SessionPanelState};
pub use slash::{SlashCommand, SlashCommandError};
pub use status_panel::StatusPanelState;
pub use task_panel::{TaskPanelNotification, TaskPanelState};
pub use terminal::{
    CrosstermTerminalBackend, FixtureTerminalBackend, TerminalModeBackend, TerminalModeError,
    TerminalModeGuard,
};
pub use trace_panel::{TraceEntryKind, TraceFilter, TracePanelEntry, TracePanelState};
pub use transcript::{TranscriptItem, TranscriptItemKind, TranscriptState};

#[must_use]
pub const fn launch_message() -> &'static str {
    "agent tui shell ready\n"
}
