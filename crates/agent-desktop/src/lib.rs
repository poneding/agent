#![forbid(unsafe_code)]
//! Desktop presentation contracts for agent.

pub mod conversation;
pub mod diff;
pub mod error;
pub mod extension;
pub mod permission;
pub mod session;
pub mod settings;
pub mod snapshot;
pub mod task;
pub mod trace;

pub use conversation::DesktopConversationItem;
pub use diff::{
    DiffFileSummary, DiffReview, DiffReviewStatus, approve_diff_review, reject_diff_review,
};
pub use error::DesktopError;
pub use extension::{DesktopExtensionHealth, DesktopExtensionStatus};
pub use permission::{
    DesktopPermissionDecision, PermissionDialog, PermissionDialogStatus, expire_permission_dialog,
    resolve_permission_dialog,
};
pub use session::{
    DesktopRoute, DesktopSession, DesktopSessionState, disconnect_desktop_session,
    start_desktop_session, switch_desktop_route,
};
pub use settings::{
    DesktopConfigSnapshot, DesktopSettings, DesktopTheme, McpSettings, MemorySettings,
    ModelSettings, PolicySettings, UiSettings, validate_desktop_settings,
};
pub use snapshot::{DesktopWorkspaceSnapshot, build_workspace_snapshot};
pub use task::DesktopTaskItem;
pub use trace::{TraceTimeline, TraceTimelineItem, TraceTimelineSeverity, build_trace_timeline};
