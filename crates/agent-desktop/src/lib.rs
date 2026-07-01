#![forbid(unsafe_code)]
//! Desktop presentation contracts for agent.

pub mod conversation;
pub mod dashboard;
pub mod diff;
pub mod error;
pub mod extension;
pub mod permission;
pub mod session;
pub mod settings;
pub mod snapshot;
pub mod task;
pub mod trace;
pub mod update;

pub use conversation::{
    DesktopBridgeErrorNotice, DesktopConversationDiagnostic, DesktopConversationDiagnosticSeverity,
    DesktopConversationFrameUpdate, DesktopConversationItem, DesktopConversationPanel,
    DesktopTranscriptEntry, DesktopTranscriptRole, DesktopTurnCommandAck, DesktopTurnSubmission,
    DesktopTurnSubmissionStatus, conversation_update_from_frame,
};
pub use dashboard::{
    DesktopDashboardDiagnostic, DesktopDashboardDiagnosticSeverity, DesktopDashboardInput,
    DesktopDashboardSnapshot, DesktopProviderReadiness, DesktopSessionReadiness,
    DesktopSessionReadinessState, DesktopTaskCounts, build_dashboard_snapshot,
};
pub use diff::{
    DesktopDiffFileReview, DesktopDiffHunkPreview, DesktopDiffLineKind, DesktopDiffLinePreview,
    DesktopDiffReview, DiffFileSummary, DiffReview, DiffReviewStatus, approve_diff_review,
    reject_diff_review,
};
pub use error::DesktopError;
pub use extension::{DesktopExtensionHealth, DesktopExtensionStatus};
pub use permission::{
    DesktopApprovalPreview, DesktopApprovalReview, DesktopApprovalReviewStatus,
    DesktopPermissionDecision, PermissionDialog, PermissionDialogStatus, expire_permission_dialog,
    resolve_permission_dialog,
};
pub use session::{
    DesktopRoute, DesktopSession, DesktopSessionState, disconnect_desktop_session,
    start_desktop_session, switch_desktop_route,
};
pub use settings::{
    DesktopConfigSnapshot, DesktopReleaseSmokeSnapshot, DesktopSettings, DesktopTheme, McpSettings,
    MemorySettings, ModelSettings, PolicySettings, UiSettings,
    build_desktop_release_smoke_snapshot, validate_desktop_settings,
};
pub use snapshot::{DesktopWorkspaceSnapshot, build_workspace_snapshot};
pub use task::{DesktopTaskArtifactItem, DesktopTaskItem, DesktopTaskReviewItem};
pub use trace::{TraceTimeline, TraceTimelineItem, TraceTimelineSeverity, build_trace_timeline};
pub use update::{
    DesktopUpdateArtifact, DesktopUpdateDecision, UpdateManifest, parse_update_manifest_json,
    validate_update_manifest, validate_update_manifest_for_current,
};
