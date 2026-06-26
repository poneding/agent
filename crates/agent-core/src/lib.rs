#![forbid(unsafe_code)]
//! Core runtime orchestration for agent.

pub mod audit_export;
pub mod benchmark;
pub mod config;
pub mod context;
pub mod idempotency;
pub mod local_coding;
pub mod model_runtime;
pub mod recovery;
pub mod runtime;
pub mod tool_loop;
pub mod trace;
pub mod turn;

pub use audit_export::{
    AuditEventCategory, AuditExportBundle, AuditExportError, AuditExportOptions, AuditExportRecord,
    TraceExportSummary, export_audit_bundle, export_audit_bundle_at,
};
pub use benchmark::{
    BenchmarkEnvironment, BenchmarkError, BenchmarkResult, BenchmarkRunner, BenchmarkScenario,
    run_smoke_benchmarks, smoke_benchmark_scenarios,
};
pub use config::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, AgentRuntimeConfig, ConfigDiagnostic,
    ConfigDiagnosticSeverity, ConfigLayer, ConfigLayerKind, ConfigLayerSummary, ConfigOverride,
    EffectiveRuntimeConfig, RuntimeModelConfig, build_effective_runtime_config,
    load_agent_config_file, load_agent_config_toml,
};
pub use context::ContextAssembler;
pub use idempotency::{IdempotentToolError, IdempotentToolExecutor, IdempotentToolOutcome};
pub use local_coding::{
    LocalCodingError, LocalCodingOutcome, LocalCodingRun, replay_local_coding_outcome,
    run_local_coding_fixture, run_local_coding_fixture_with_shell_denied,
};
pub use model_runtime::{
    ConfiguredModelRuntime, ConfiguredTurnError, ConfiguredTurnInput, ConfiguredTurnOutcome,
    ConfiguredTurnRunner, ModelInvocationAttempt, ModelInvocationError, ModelInvocationOutcome,
    ModelInvocationRunner, ModelRuntimeError, ModelRuntimeResolver,
};
pub use recovery::{RecoveryEngine, RecoveryError, RecoveryOutcome, RecoveryPlan};
pub use runtime::RuntimeIds;
pub use tool_loop::{ToolCallRequest, ToolLoop, ToolLoopError, ToolLoopOutcome, ToolLoopStatus};
pub use trace::{TraceEntry, TraceError, TraceQuery, build_trace};
pub use turn::{TurnEngine, TurnError, TurnInput, TurnOutcome};

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
