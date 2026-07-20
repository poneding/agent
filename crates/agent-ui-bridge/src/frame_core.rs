//! Core UI frame surface for the terminal daily-driver path.
//!
//! Interactive TUI hosts should treat core variants as the hot path and treat
//! [`crate::UiJsonFrame::Admin`] as management/admin traffic that must not block
//! chat UX work.

use crate::UiJsonFrame;

/// Frame kinds the interactive TUI must handle for a coding agent session.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CoreUiFrameKind {
    Event,
    ConfigSnapshot,
    TerminalSessionSnapshot,
    ApprovalQueueSnapshot,
    SlashCommandCatalogSnapshot,
    TerminalStatusSnapshot,
    EditDiffSnapshot,
    ModelSelection,
    ModelInvocationAttempts,
    PlanTodoSnapshot,
    ProjectContextSnapshot,
    ProjectContextDetail,
    Done,
    Error,
}

impl CoreUiFrameKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::ConfigSnapshot => "config_snapshot",
            Self::TerminalSessionSnapshot => "terminal_session_snapshot",
            Self::ApprovalQueueSnapshot => "approval_queue_snapshot",
            Self::SlashCommandCatalogSnapshot => "slash_command_catalog_snapshot",
            Self::TerminalStatusSnapshot => "terminal_status_snapshot",
            Self::EditDiffSnapshot => "edit_diff_snapshot",
            Self::ModelSelection => "model_selection",
            Self::ModelInvocationAttempts => "model_invocation_attempts",
            Self::PlanTodoSnapshot => "plan_todo_snapshot",
            Self::ProjectContextSnapshot => "project_context_snapshot",
            Self::ProjectContextDetail => "project_context_detail",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

/// Returns true when a frame belongs to the interactive coding hot path.
#[must_use]
pub fn is_core_ui_frame(frame: &UiJsonFrame) -> bool {
    matches!(
        frame,
        UiJsonFrame::Event(_)
            | UiJsonFrame::ConfigSnapshot(_)
            | UiJsonFrame::TerminalSessionSnapshot(_)
            | UiJsonFrame::ApprovalQueueSnapshot(_)
            | UiJsonFrame::SlashCommandCatalogSnapshot(_)
            | UiJsonFrame::TerminalStatusSnapshot(_)
            | UiJsonFrame::EditDiffSnapshot(_)
            | UiJsonFrame::ModelSelection { .. }
            | UiJsonFrame::ModelInvocationAttempts(_)
            | UiJsonFrame::ConfigSummary { .. }
            | UiJsonFrame::ConfigDiagnostic { .. }
            | UiJsonFrame::PlanTodoSnapshot(_)
            | UiJsonFrame::ProjectContextSnapshot(_)
            | UiJsonFrame::ProjectContextDetail(_)
            | UiJsonFrame::Done { .. }
            | UiJsonFrame::Error { .. }
    )
}

/// Classify a frame for hosts that want to demote non-core snapshots.
#[must_use]
pub fn core_ui_frame_kind(frame: &UiJsonFrame) -> Option<CoreUiFrameKind> {
    match frame {
        UiJsonFrame::Event(_) => Some(CoreUiFrameKind::Event),
        UiJsonFrame::ConfigSnapshot(_) => Some(CoreUiFrameKind::ConfigSnapshot),
        UiJsonFrame::TerminalSessionSnapshot(_) => Some(CoreUiFrameKind::TerminalSessionSnapshot),
        UiJsonFrame::ApprovalQueueSnapshot(_) => Some(CoreUiFrameKind::ApprovalQueueSnapshot),
        UiJsonFrame::SlashCommandCatalogSnapshot(_) => {
            Some(CoreUiFrameKind::SlashCommandCatalogSnapshot)
        }
        UiJsonFrame::TerminalStatusSnapshot(_) => Some(CoreUiFrameKind::TerminalStatusSnapshot),
        UiJsonFrame::EditDiffSnapshot(_) => Some(CoreUiFrameKind::EditDiffSnapshot),
        UiJsonFrame::ModelSelection { .. } => Some(CoreUiFrameKind::ModelSelection),
        UiJsonFrame::ModelInvocationAttempts(_) => Some(CoreUiFrameKind::ModelInvocationAttempts),
        UiJsonFrame::PlanTodoSnapshot(_) => Some(CoreUiFrameKind::PlanTodoSnapshot),
        UiJsonFrame::ProjectContextSnapshot(_) => Some(CoreUiFrameKind::ProjectContextSnapshot),
        UiJsonFrame::ProjectContextDetail(_) => Some(CoreUiFrameKind::ProjectContextDetail),
        UiJsonFrame::Done { .. } => Some(CoreUiFrameKind::Done),
        UiJsonFrame::Error { .. } => Some(CoreUiFrameKind::Error),
        UiJsonFrame::Admin(_)
        | UiJsonFrame::ConfigSummary { .. }
        | UiJsonFrame::ConfigDiagnostic { .. } => None,
    }
}
