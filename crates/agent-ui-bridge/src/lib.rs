#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.

pub mod command;
pub mod error;
pub mod event;
pub mod frame;

pub use command::{
    UiApprovalDecision, UiClientInfo, UiCommand, UiCommandEnvelope, UiPermissionDecision,
    validate_ui_command,
};
pub use error::UiBridgeError;
pub use event::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiDiagnosticLevel, UiEventEnvelope,
    UiEventPayload, UiTaskNotification, normalize_runtime_event,
};
pub use frame::{
    UiCommandTestSnapshot, UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot,
    UiEditDiffSnapshot, UiExtensionCapabilityReview, UiExtensionDiagnostic, UiExtensionSnapshot,
    UiExtensionSummary, UiFinalReviewCommandResult, UiFinalReviewSnapshot, UiJsonFrame,
    UiMcpManagementDiagnostic, UiMcpManagementSnapshot, UiMcpServerManagementSummary,
    UiModelCostProfile, UiModelProviderCapabilities, UiModelProviderSummary,
    UiObservabilitySnapshot, UiPlatformPathsSummary, UiProjectContextDiagnosticSummary,
    UiProjectContextFileSummary, UiProjectContextOmissionSummary, UiProjectContextSnapshot,
    UiPromptDiagnosticSummary, UiPromptLayerSummary, UiPromptSummary, UiProviderSupportRole,
    UiProviderSupportSummary, UiRepoSessionDiagnostic, UiRepoSessionSnapshot,
    UiSettingsProfileSummary, UiStartupMigrationSnapshot, UiTaskCommandError, UiTaskReviewSnapshot,
    UiTaskReviewSummary, UiTaskReviewSummaryArtifact, UiTaskScheduleSummary, UiTaskSnapshot,
    UiTaskSummary, encode_json_frame,
};
