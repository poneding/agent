#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.

pub mod approval_queue;
pub mod command;
pub mod error;
pub mod event;
pub mod frame;
pub mod frame_core;
pub mod slash;
pub mod transcript_kind;

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
    UiEventPayload, UiTaskNotification, UiToolPolicySummary, UiToolSandboxSummary,
    normalize_runtime_event,
};
pub use frame::{
    UiAdminFrame, UiCommandRetryPlan, UiCommandTestSnapshot, UiConfigDiagnostic,
    UiConfigDiagnosticSeverity, UiConfigSnapshot, UiConflictInspectReport,
    UiContextQualityDiagnosticSummary, UiContextQualitySignalSummary,
    UiConversationCompactionReport, UiEditDiffSnapshot, UiExtensionCapabilityReview,
    UiExtensionDiagnostic, UiExtensionSnapshot, UiExtensionSummary, UiFinalReviewCommandResult,
    UiFinalReviewSnapshot, UiIdempotentRecoverySummary, UiJsonFrame, UiMcpManagementDiagnostic,
    UiMcpManagementSnapshot, UiMcpServerManagementSummary, UiMemoryDiagnostic,
    UiMemoryInfluenceEntrySummary, UiMemoryInfluenceSummary, UiMemoryProposalSummary,
    UiMemoryRecordSummary, UiMemoryRetentionSummary, UiMemoryRetrievalCandidateSummary,
    UiMemoryRetrievalPreview, UiMemorySnapshot, UiModelCostProfile, UiModelInvocationAttempt,
    UiModelInvocationAttemptSnapshot, UiModelProviderCapabilities, UiModelProviderSummary,
    UiObservabilitySnapshot, UiOnboardingDiagnostic, UiOnboardingNextAction,
    UiOnboardingReadinessSnapshot, UiPinnedContextItemSummary, UiPinnedContextOmissionSummary,
    UiPlanTodoCommandError, UiPlanTodoSnapshot, UiPlatformPathsSummary, UiProjectContextDetail,
    UiProjectContextDiagnosticSummary, UiProjectContextFileSummary,
    UiProjectContextHybridCandidateSummary, UiProjectContextHybridOmissionSummary,
    UiProjectContextHybridRetrievalSnapshot, UiProjectContextOmissionSummary,
    UiProjectContextPinnedRetrievalSnapshot, UiProjectContextQualityTraceSnapshot,
    UiProjectContextRetrievalCandidateSummary, UiProjectContextRetrievalDiagnosticSummary,
    UiProjectContextRetrievalOmissionSummary, UiProjectContextRetrievalScore,
    UiProjectContextRetrievalSnapshot, UiProjectContextSemanticRetrievalSummary,
    UiProjectContextSnapshot, UiPromptDiagnosticSummary, UiPromptLayerSummary, UiPromptSummary,
    UiProviderSupportRole, UiProviderSupportSummary, UiRecoverySnapshot, UiReplayForkReport,
    UiReplayInspectReport, UiRepoSessionDiagnostic, UiRepoSessionSnapshot, UiResumeDryRunReport,
    UiResumeReport, UiSettingsProfileSummary, UiStartupMigrationSnapshot, UiTaskActiveTodoSummary,
    UiTaskCommandError, UiTaskReviewSnapshot, UiTaskReviewSummary, UiTaskReviewSummaryArtifact,
    UiTaskScheduleSummary, UiTaskSnapshot, UiTaskSummary, UiTeamWorkflowCommandError,
    UiTeamWorkflowSnapshot, UiTerminalSessionDiagnostic, UiTerminalSessionSnapshot,
    UiTerminalSessionSummary, encode_json_frame,
};
pub use frame_core::{CoreUiFrameKind, core_ui_frame_kind, is_core_ui_frame};
pub use slash::{
    UiSlashCommandCatalogSnapshot, UiSlashCommandCategory, UiSlashCommandDescriptor,
    UiSlashCommandEffect, UiSlashCommandParseError, UiSlashCommandSurface,
    UiTerminalStatusSnapshot, default_slash_command_catalog, resolve_slash_command,
    resolve_slash_command_in_catalog,
};
pub use transcript_kind::UiTranscriptKind;
