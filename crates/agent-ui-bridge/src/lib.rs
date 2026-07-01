#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.

pub mod approval_queue;
pub mod command;
pub mod error;
pub mod event;
pub mod frame;
pub mod slash;

pub use approval_queue::{
    UiApprovalQueueDiagnostic, UiApprovalQueueDiagnosticSeverity, UiApprovalQueueError,
    UiApprovalQueueGroup, UiApprovalQueueItem, UiApprovalQueueSnapshot,
    approval_queue_snapshot_from_events, approval_queue_snapshot_from_requests,
};
pub use command::{
    UiApprovalDecision, UiClientInfo, UiCommand, UiCommandEnvelope, UiPermissionDecision,
    UiPlanTodoCommand, validate_ui_command,
};
pub use error::UiBridgeError;
pub use event::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiDiagnosticLevel, UiEventEnvelope,
    UiEventPayload, UiTaskNotification, normalize_runtime_event,
};
pub use frame::{
    UiCommandTestSnapshot, UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot,
    UiContextQualityDiagnosticSummary, UiContextQualitySignalSummary, UiEditDiffSnapshot,
    UiExtensionCapabilityReview, UiExtensionDiagnostic, UiExtensionSnapshot, UiExtensionSummary,
    UiFinalReviewCommandResult, UiFinalReviewSnapshot, UiJsonFrame, UiMcpManagementDiagnostic,
    UiMcpManagementSnapshot, UiMcpServerManagementSummary, UiModelCostProfile,
    UiModelProviderCapabilities, UiModelProviderSummary, UiObservabilitySnapshot,
    UiPinnedContextItemSummary, UiPinnedContextOmissionSummary, UiPlanTodoCommandError,
    UiPlanTodoSnapshot, UiPlatformPathsSummary, UiProjectContextDiagnosticSummary,
    UiProjectContextFileSummary, UiProjectContextHybridCandidateSummary,
    UiProjectContextHybridOmissionSummary, UiProjectContextHybridRetrievalSnapshot,
    UiProjectContextOmissionSummary, UiProjectContextPinnedRetrievalSnapshot,
    UiProjectContextQualityTraceSnapshot, UiProjectContextRetrievalCandidateSummary,
    UiProjectContextRetrievalDiagnosticSummary, UiProjectContextRetrievalOmissionSummary,
    UiProjectContextRetrievalScore, UiProjectContextRetrievalSnapshot,
    UiProjectContextSemanticRetrievalSummary, UiProjectContextSnapshot, UiPromptDiagnosticSummary,
    UiPromptLayerSummary, UiPromptSummary, UiProviderSupportRole, UiProviderSupportSummary,
    UiRepoSessionDiagnostic, UiRepoSessionSnapshot, UiSettingsProfileSummary,
    UiStartupMigrationSnapshot, UiTaskActiveTodoSummary, UiTaskCommandError, UiTaskReviewSnapshot,
    UiTaskReviewSummary, UiTaskReviewSummaryArtifact, UiTaskScheduleSummary, UiTaskSnapshot,
    UiTaskSummary, UiTerminalSessionDiagnostic, UiTerminalSessionSnapshot,
    UiTerminalSessionSummary, encode_json_frame,
};
pub use slash::{
    UiSlashCommandCatalogSnapshot, UiSlashCommandCategory, UiSlashCommandDescriptor,
    UiSlashCommandEffect, UiSlashCommandParseError, UiSlashCommandSurface,
    UiTerminalStatusSnapshot, default_slash_command_catalog, resolve_slash_command,
    resolve_slash_command_in_catalog,
};
