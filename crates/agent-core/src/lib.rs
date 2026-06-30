#![forbid(unsafe_code)]
//! Core runtime orchestration for agent.

pub mod approval;
pub mod audit_export;
pub mod background;
pub mod benchmark;
pub mod command_test;
pub mod config;
pub mod context;
pub mod edit_diff;
pub mod extension;
pub mod final_review;
pub mod idempotency;
pub mod local_coding;
pub mod mcp_sampling;
pub mod model_runtime;
pub mod observability;
pub mod project_context;
pub mod provider_support;
pub mod recovery;
pub mod repo_session;
pub mod runtime;
pub mod settings;
pub mod startup_migration;
pub mod task_review;
pub mod tool_loop;
pub mod trace;
pub mod turn;

pub use approval::{
    ApprovalCoordinator, ApprovalRecoveryView, ApprovalResumeError, ApprovalResumeOutcome,
    ApprovalResumeStatus, SuspendedToolApproval,
};
pub use audit_export::{
    AuditEventCategory, AuditExportBundle, AuditExportError, AuditExportOptions, AuditExportRecord,
    TraceExportSummary, export_audit_bundle, export_audit_bundle_at,
};
pub use background::{
    BackgroundApprovalDecision, BackgroundApprovalDecisionKind, BackgroundRunnerError,
    BackgroundTaskRunOutcome, BackgroundTaskRunner, BackgroundTaskRunnerInput,
    BackgroundTaskTurnError, BackgroundTaskTurnExecutor, BackgroundTaskTurnResult,
    ComprehensiveBackgroundTurnExecutor,
};
pub use benchmark::{
    BenchmarkEnvironment, BenchmarkError, BenchmarkResult, BenchmarkRunner, BenchmarkScenario,
    run_smoke_benchmarks, smoke_benchmark_scenarios,
};
pub use command_test::{
    BoundedCommandOutput, CommandCancellationState, CommandExecutionStatus,
    CommandFailureClassification, CommandIntentSummary, CommandObservationSummary,
    CommandTestDiagnostic, CommandTestDiagnosticSeverity, CommandTestLoopError,
    CommandTestLoopInput, CommandTestLoopReport, run_command_test_loop,
};
pub use config::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, AgentRuntimeConfig, ConfigDiagnostic,
    ConfigDiagnosticSeverity, ConfigLayer, ConfigLayerKind, ConfigLayerSummary, ConfigOverride,
    EffectiveRuntimeConfig, RuntimeModelConfig, build_effective_runtime_config,
    load_agent_config_file, load_agent_config_toml,
};
pub use context::ContextAssembler;
pub use edit_diff::{
    DiffEvidenceState, DiffSummary, EditDiffDiagnostic, EditDiffDiagnosticSeverity,
    EditDiffFlowError, EditDiffFlowInput, EditDiffFlowReport, EditPreviewSummary,
    run_edit_diff_flow,
};
pub use extension::{
    ExtensionCapabilityLease, ExtensionCapabilitySummary, ExtensionCompatibility, ExtensionHealth,
    ExtensionHealthStatus, ExtensionId, ExtensionKind, ExtensionPermissionSummary, ExtensionRecord,
    ExtensionRegistry, ExtensionRegistryError, ExtensionRegistrySnapshot, ExtensionSource,
    ExtensionSourceKind,
};
pub use final_review::{
    CodingFinalReview, FinalReviewCommandResult, FinalReviewDiagnostic,
    FinalReviewDiagnosticSeverity, FinalReviewError, FinalReviewInput, build_final_review,
    replay_final_review,
};
pub use idempotency::{IdempotentToolError, IdempotentToolExecutor, IdempotentToolOutcome};
pub use local_coding::{
    LocalCodingError, LocalCodingOutcome, LocalCodingRun, replay_local_coding_outcome,
    run_local_coding_fixture, run_local_coding_fixture_with_shell_denied,
};
pub use mcp_sampling::{
    McpModelSamplingProvider, McpSamplingBridgeDiagnostic, McpSamplingModelBridgeConfig,
    McpSamplingModelBridgeError, McpSamplingModelBridgeIds, build_mcp_sampling_model_request,
    mcp_sampling_bridge_error_diagnostic, mcp_sampling_model_error_diagnostic,
};
pub use model_runtime::{
    AgentLoopBudget, AgentLoopFailureReason, AgentLoopOutcome, AgentLoopTerminalState,
    ComprehensiveTurnInput, ComprehensiveTurnResources, ComprehensiveTurnRunner,
    ConfiguredModelRuntime, ConfiguredTurnError, ConfiguredTurnEventStream, ConfiguredTurnInput,
    ConfiguredTurnOutcome, ConfiguredTurnPersistenceError, ConfiguredTurnRunner,
    ConfiguredTurnStream, ConfiguredTurnStreamOptions, ModelInvocationAttempt,
    ModelInvocationError, ModelInvocationOutcome, ModelInvocationRunner, ModelRuntimeError,
    ModelRuntimeResolver,
};
pub use observability::{
    EvidenceReplayVerificationExpectation, EvidenceReplayVerificationReport,
    OBSERVABILITY_EVIDENCE_SCHEMA_VERSION, ObservabilityEvidenceBundle, ObservabilityEvidenceError,
    ObservabilityEvidenceOptions, ObservabilityEvidenceSummary, ReplayEvidenceSummary,
    build_observability_evidence, build_observability_evidence_at,
    verify_observability_evidence_replay, verify_observability_replay_fixture,
};
pub use project_context::{
    ContextCandidate, ContextCandidateOmission, ContextCandidateOmissionReason,
    ContextHybridCandidate, ContextHybridCandidateOmission, ContextHybridRetrievalReport,
    ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery,
    ContextRetrievalReport, ContextSelectionReason, ContextSemanticFallbackPolicy,
    ContextSemanticRetrievalConfig, ContextSemanticRetrievalMode, ContextSemanticRetrievalStatus,
    ContextSemanticRetrievalSummary, ProjectContextDiagnostic, ProjectContextDiagnosticSeverity,
    ProjectContextLimits, ProjectContextOmission, ProjectContextSummary, ProjectContextVisibility,
    ProjectDirectorySummary, ProjectFileKind, ProjectFileSummary, ProjectSemanticDocument,
};
pub use project_context::{
    ProjectContextHybridRetrievalInput, ProjectContextHybridRetrievalReport,
    ProjectContextRetrievalError, ProjectContextRetrievalInput, ProjectContextRetrievalReport,
    ProjectContextScanError, ProjectContextScanInput, ProjectContextScanReport,
    retrieve_project_context, retrieve_project_context_hybrid, scan_project_context,
};
pub use provider_support::{
    ProviderSupportReport, ProviderSupportRole, ProviderSupportSummary,
    coding_agent_model_requirements,
};
pub use recovery::{RecoveryEngine, RecoveryError, RecoveryOutcome, RecoveryPlan};
pub use repo_session::{
    RepoConfigSummary, RepoGitState, RepoGitSummary, RepoProviderSummary,
    RepoSessionBootstrapError, RepoSessionBootstrapInput, RepoSessionBootstrapReport,
    RepoSessionDiagnostic, RepoSessionDiagnosticSeverity, bootstrap_repo_session,
};
pub use runtime::RuntimeIds;
pub use settings::{
    PlatformKind, PlatformPathKey, PlatformPathOverrides, PlatformPaths, ReleaseChannel,
    SETTINGS_SCHEMA_VERSION, SettingsError, SettingsLoadResult, SettingsMutation, SettingsProfile,
    load_settings_file, write_settings_file_atomic,
};
pub use startup_migration::{
    NoopStartupMigrationParticipant, SchemaVersionRecord, StartupMigrationAuditRecord,
    StartupMigrationAuditStatus, StartupMigrationDiagnostic, StartupMigrationDiagnosticSeverity,
    StartupMigrationError, StartupMigrationOutcome, StartupMigrationParticipant,
    StartupMigrationPlan, StartupMigrationRunner, StartupMigrationStep,
    StorageJsonStartupMigrationParticipant, default_startup_migration_participants,
    startup_migration_status_json,
};
pub use task_review::{TaskArtifact, TaskArtifactKind, TaskReviewError, TaskReviewSummary};
pub use tool_loop::{
    AccumulatedToolCall, ToolCallAccumulator, ToolCallRequest, ToolLoop, ToolLoopError,
    ToolLoopOutcome, ToolLoopStatus, ToolObservationMessage,
};
pub use trace::{TraceEntry, TraceError, TraceQuery, build_trace};
pub use turn::{
    CancellationToken, TurnEngine, TurnError, TurnEventStream, TurnInput, TurnOutcome,
    TurnStreamOptions,
};

/// Returns a smoke-test banner proving the core crate links model and storage.
#[must_use]
pub fn runtime_banner() -> String {
    format!(
        "agent ({}, {})",
        agent_model::model_layer_name(),
        agent_storage::storage_layer_name()
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_runtime_banner() {
        assert_eq!(
            crate::runtime_banner(),
            "agent (agent-model uses agent-types, agent-storage uses agent-types)"
        );
    }
}
