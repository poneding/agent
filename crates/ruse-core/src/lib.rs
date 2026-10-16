#![forbid(unsafe_code)]
//! Core runtime orchestration for agent.
//!
//! Interactive hosts should start from [`AgentSession`] (`prompt` /
//! `prompt_with_tools`) rather than wiring
//! `ConfiguredTurnRunner` / `ComprehensiveTurnRunner` by hand.

pub mod approval;
pub mod audit_export;
pub mod background;
pub mod benchmark;
pub mod command_retry;
pub mod command_test;
pub mod config;
pub mod conflict_recovery;
pub mod context;
pub mod edit_diff;
pub mod extension;
pub mod final_review;
pub mod idempotency;
pub mod local_coding;
#[cfg(feature = "mcp")]
pub mod mcp_sampling;
pub mod memory;
pub mod model_runtime;
pub mod observability;
pub mod onboarding;
pub mod plan;
pub mod prelude;
pub mod project_context;
pub mod provider_support;
pub mod recovery;
pub mod recovery_smoke;
pub mod repo_session;
pub mod runtime;
pub mod session;
pub mod settings;
pub mod startup_migration;
pub mod task_review;
pub mod terminal_session;
pub mod text_tool_call;
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
pub use command_retry::{
    CommandRetryExecutionInput, CommandRetryExecutionReport, CommandRetryPlanner,
};
pub use command_test::{
    BoundedCommandOutput, CommandCancellationState, CommandExecutionStatus,
    CommandFailureClassification, CommandIntentSummary, CommandObservationSummary,
    CommandTestDiagnostic, CommandTestDiagnosticSeverity, CommandTestLoopError,
    CommandTestLoopInput, CommandTestLoopReport, DEFAULT_COMMAND_TEST_TIMEOUT_MS,
    run_command_test_loop,
};
pub use config::{
    AGENT_CONFIG_FILE_NAME, AGENT_CONFIG_SCHEMA_VERSION, AGENT_LEGACY_CONFIG_FILE_NAME,
    AgentConfigError, AgentConfigPathResolution, AgentRuntimeConfig, ConfigDiagnostic,
    ConfigDiagnosticSeverity, ConfigLayer, ConfigLayerKind, ConfigLayerSummary, ConfigOverride,
    EffectiveRuntimeConfig, LoadedAgentConfig, RuntimeModelConfig, RuntimeTerminalConfig,
    RuntimeTerminalTheme, build_effective_runtime_config, default_agent_config_path,
    default_agent_config_path_for_home, legacy_agent_config_path,
    legacy_agent_config_path_for_home, load_agent_config_file, load_agent_config_file_with_secrets,
    load_agent_config_toml, load_agent_config_toml_with_secrets, load_agent_config_yaml,
    load_agent_config_yaml_with_secrets, resolve_agent_config_path,
    resolve_agent_config_path_for_home, write_agent_config_terminal_theme_atomic,
};
pub use conflict_recovery::{ConflictRecoveryMapper, conflict_diagnostic};
pub use context::{
    ContextAssembler, ConversationCompactionInput, ConversationCompactionReport,
    coding_context_assembler, compact_conversation_events, conversation_history_messages,
    load_workspace_git_summary_layer, load_workspace_instruction_layer,
};
pub use edit_diff::{
    DiffEvidenceState, DiffSummary, EditDiffDiagnostic, EditDiffDiagnosticSeverity,
    EditDiffFlowError, EditDiffFlowInput, EditDiffFlowReport, EditPreviewSummary,
    run_edit_diff_flow,
};
pub use extension::{
    EXTENSION_HEALTH_SMOKE_SCHEMA_VERSION, ExtensionCapabilityLease, ExtensionCapabilitySummary,
    ExtensionCompatibility, ExtensionHealth, ExtensionHealthKindCount,
    ExtensionHealthSmokeDiagnostic, ExtensionHealthSmokeDiagnosticSeverity,
    ExtensionHealthSmokeReport, ExtensionHealthStatus, ExtensionHealthStatusCount, ExtensionId,
    ExtensionKind, ExtensionPermissionSummary, ExtensionRecord, ExtensionRegistry,
    ExtensionRegistryError, ExtensionRegistrySnapshot, ExtensionSource, ExtensionSourceKind,
    MAX_EXTENSION_HEALTH_SMOKE_DIAGNOSTICS, MAX_EXTENSION_HEALTH_SMOKE_MESSAGE_CHARS,
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
#[cfg(feature = "mcp")]
pub use mcp_sampling::{
    McpModelSamplingProvider, McpSamplingBridgeDiagnostic, McpSamplingModelBridgeConfig,
    McpSamplingModelBridgeError, McpSamplingModelBridgeIds, build_mcp_sampling_model_request,
    mcp_sampling_bridge_error_diagnostic, mcp_sampling_model_error_diagnostic,
};
pub use memory::{
    RuntimeMemoryAssembly, RuntimeMemoryConfig, RuntimeMemoryDiagnostic,
    RuntimeMemoryDiagnosticSeverity, RuntimeMemoryError, RuntimeMemoryInfluenceReport,
    RuntimeMemoryInput, RuntimeMemoryProposalInput, RuntimeMemoryProposalPolicy,
    RuntimeMemoryProposalReport, RuntimeMemoryTurnEvidence, assemble_runtime_memory,
};
pub use model_runtime::{
    AgentLoopBudget, AgentLoopFailureReason, AgentLoopOutcome, AgentLoopTerminalState,
    ComprehensiveApprovalContinuation, ComprehensiveTurnInput, ComprehensiveTurnResources,
    ComprehensiveTurnRunner, ComprehensiveTurnStreamOptions, ConfiguredModelRuntime,
    ConfiguredTurnError, ConfiguredTurnEventStream, ConfiguredTurnInput, ConfiguredTurnOutcome,
    ConfiguredTurnPersistenceError, ConfiguredTurnRunner, ConfiguredTurnStream,
    ConfiguredTurnStreamOptions, ModelInvocationAttempt, ModelInvocationError,
    ModelInvocationOutcome, ModelInvocationRunner, ModelRuntimeError, ModelRuntimeResolver,
    PendingComprehensiveApproval,
};
pub use observability::{
    EvidenceReplayVerificationExpectation, EvidenceReplayVerificationReport,
    OBSERVABILITY_EVIDENCE_SCHEMA_VERSION, ObservabilityEvidenceBundle, ObservabilityEvidenceError,
    ObservabilityEvidenceOptions, ObservabilityEvidenceSummary, ReplayEvidenceSummary,
    build_observability_evidence, build_observability_evidence_at,
    verify_observability_evidence_replay, verify_observability_replay_fixture,
};
pub use onboarding::{
    ONBOARDING_READINESS_SCHEMA_VERSION, OnboardingDiagnostic, OnboardingDiagnosticSeverity,
    OnboardingMigrationSummary, OnboardingNextAction, OnboardingNextActionTarget,
    OnboardingPathSummary, OnboardingProviderSummary, OnboardingReadinessInput,
    OnboardingReadinessSnapshot, OnboardingReadinessStatus, OnboardingSecretStoreSummary,
    OnboardingSettingsSummary,
};
pub use plan::{
    PlanTodoError, PlanTodoOperation, PlanTodoUpdate, PlanTodoUpdateOutcome,
    append_task_review_evidence, apply_plan_todo_update, normalize_plan_todo_snapshot,
    validate_plan_todo_snapshot,
};
pub use project_context::{
    ContextBudget, ContextCandidate, ContextCandidateOmission, ContextCandidateOmissionReason,
    ContextHybridCandidate, ContextHybridCandidateOmission, ContextHybridRetrievalReport,
    ContextQualityBudgetPressure, ContextQualityDiagnostic, ContextQualityDiagnosticSeverity,
    ContextQualityRetrievalKind, ContextQualitySignal, ContextQualitySignalKind,
    ContextQualityTrace, ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity,
    ContextRetrievalQuery, ContextRetrievalReport, ContextSelectionReason,
    ContextSemanticFallbackPolicy, ContextSemanticRetrievalConfig, ContextSemanticRetrievalMode,
    ContextSemanticRetrievalStatus, ContextSemanticRetrievalSummary, PinnedContextItem,
    PinnedContextOmission, PinnedContextOmissionReason, PinnedContextPriority, PinnedContextSource,
    PinnedProjectContextBudgetReport, PinnedProjectContextPin, PinnedProjectContextRequest,
    ProjectContextDiagnostic, ProjectContextDiagnosticSeverity, ProjectContextLimits,
    ProjectContextOmission, ProjectContextSummary, ProjectContextVisibility,
    ProjectDirectorySummary, ProjectFileKind, ProjectFileSummary, ProjectSemanticDocument,
};
pub use project_context::{
    ProjectContextHybridPinnedQualityRetrievalReport, ProjectContextHybridPinnedRetrievalInput,
    ProjectContextHybridPinnedRetrievalReport, ProjectContextHybridQualityRetrievalReport,
    ProjectContextHybridRetrievalInput, ProjectContextHybridRetrievalReport,
    ProjectContextPinnedQualityRetrievalReport, ProjectContextPinnedRetrievalInput,
    ProjectContextPinnedRetrievalReport, ProjectContextQualityRetrievalReport,
    ProjectContextRetrievalError, ProjectContextRetrievalInput, ProjectContextRetrievalReport,
    ProjectContextScanError, ProjectContextScanInput, ProjectContextScanReport,
    retrieve_project_context, retrieve_project_context_hybrid,
    retrieve_project_context_hybrid_pinned, retrieve_project_context_hybrid_pinned_quality,
    retrieve_project_context_hybrid_quality, retrieve_project_context_pinned,
    retrieve_project_context_pinned_quality, retrieve_project_context_quality,
    scan_project_context,
};
pub use provider_support::{
    ProviderSupportReport, ProviderSupportRole, ProviderSupportSummary,
    coding_agent_model_requirements,
};
pub use recovery::{RecoveryEngine, RecoveryError, RecoveryOutcome, RecoveryPlan};
pub use recovery_smoke::{
    RecoveryCrashSmokeError, RecoveryCrashSmokeReport, run_recovery_crash_smoke_fixture,
};
pub use repo_session::{
    RepoConfigSummary, RepoGitState, RepoGitSummary, RepoProviderSummary,
    RepoSessionBootstrapError, RepoSessionBootstrapInput, RepoSessionBootstrapReport,
    RepoSessionDiagnostic, RepoSessionDiagnosticSeverity, bootstrap_repo_session,
};
pub use runtime::RuntimeIds;
pub use session::{AgentSession, SessionAgentOptions, SessionPromptOptions};
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
pub use terminal_session::{
    TerminalSessionCatalog, TerminalSessionDiagnostic, TerminalSessionDiagnosticSeverity,
    TerminalSessionDirectoryError, TerminalSessionDirectoryReport, TerminalSessionRecord,
    TerminalSessionRestoreError, TerminalSessionStatus, TerminalSessionSummary,
    build_terminal_session_directory, summarize_terminal_session_title, terminal_session_status,
    validate_terminal_session_restore,
};
pub use tool_loop::{
    AccumulatedToolCall, ToolCallAccumulator, ToolCallRequest, ToolLoop, ToolLoopError,
    ToolLoopOutcome, ToolLoopStatus, ToolObservationMessage, capability_action_for_tool,
    sandbox_profile_for_tool,
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
        ruse_model::model_layer_name(),
        ruse_storage::storage_layer_name()
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_runtime_banner() {
        assert_eq!(
            crate::runtime_banner(),
            "agent (ruse-model uses ruse-types, ruse-storage uses ruse-types)"
        );
    }
}
