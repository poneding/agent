use std::sync::Arc;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentRuntimeConfig, BackgroundTaskRunner,
    BackgroundTaskRunnerInput, BackgroundTaskTurnError, BackgroundTaskTurnExecutor,
    BackgroundTaskTurnResult, ConfigLayer, ConfigLayerKind, ConfigOverride, RuntimeModelConfig,
    build_effective_runtime_config,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelEndpointConfig, ModelFallbackChain,
    ModelInvocationPolicy, ModelProviderCatalog, ModelProviderConfig, ModelProviderKind,
    ModelProviderRegistry, ModelRequestRequirements, ModelRetryPolicy, RegisteredModelProvider,
};
use agent_scheduler::{InMemoryTaskQueue, RetryPolicy, TaskKind, TaskPriority, TaskQueue, TaskRun};
use agent_types::{ConversationId, TaskState, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-27T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn task_run(summary: &str) -> TaskRun {
    TaskRun::new(
        WorkspaceId::new(),
        Some(ConversationId::new()),
        TaskKind::Background,
        TaskPriority::Normal,
        summary,
        ts(0),
    )
    .expect("valid task")
}

fn queue() -> InMemoryTaskQueue {
    InMemoryTaskQueue::new(RetryPolicy::default(), 1_000).expect("valid queue")
}

fn capabilities() -> ModelCapabilities {
    ModelCapabilities {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        max_context_tokens: Some(4096),
    }
}

fn requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        min_context_tokens: Some(512),
    }
}

fn provider_config() -> ModelProviderConfig {
    ModelProviderConfig::new(
        "fake",
        ModelProviderKind::Fake,
        "fake-background",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid provider")
}

fn invocation_policy() -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
        ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
    )
    .expect("valid policy")
}

fn loop_budget() -> AgentLoopBudget {
    AgentLoopBudget {
        max_model_turns: 2,
        max_tool_calls: 1,
        max_wall_time_ms: 30_000,
        max_observation_bytes: 4096,
        max_consecutive_invalid_tool_calls: 1,
    }
}

fn registry(config: ModelProviderConfig, script: FakeModelScript) -> ModelProviderRegistry {
    let provider = FakeModelProvider::new(script);
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(config, capabilities(), Arc::new(provider))
            .expect("registered provider"),
    ])
    .expect("registry")
}

fn effective_config(config: ModelProviderConfig) -> agent_core::EffectiveRuntimeConfig {
    let layer = ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "background-test".to_string(),
        config: AgentRuntimeConfig {
            schema_version: AGENT_CONFIG_SCHEMA_VERSION,
            model: RuntimeModelConfig {
                default_provider: "fake".to_string(),
                catalog: ModelProviderCatalog::new(vec![config]).expect("catalog"),
                invocation_policy: invocation_policy(),
            },
        },
    };
    build_effective_runtime_config(vec![layer], ConfigOverride::default()).expect("config")
}

#[tokio::test]
async fn background_turn_runner_returns_none_when_no_task_is_ready() {
    let mut queue = queue();
    let mut executor = RecordingExecutor::success("unused");

    let outcome = BackgroundTaskRunner
        .run_one(
            &mut queue,
            &mut executor,
            BackgroundTaskRunnerInput::new("worker-a", ts(1)).expect("input"),
        )
        .await
        .expect("runner");

    assert_eq!(outcome, None);
    assert_eq!(executor.executed_task_count, 0);
}

#[tokio::test]
async fn background_turn_runner_executes_fake_comprehensive_turn_and_completes_lease() {
    let config = provider_config();
    let mut queue = queue();
    let task = task_run("summarize background work");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let mut executor = agent_core::ComprehensiveBackgroundTurnExecutor::new(
        effective_config(config.clone()),
        requirements(),
        loop_budget(),
        registry(config, FakeModelScript::text("background done")),
    );

    let outcome = BackgroundTaskRunner
        .run_one(
            &mut queue,
            &mut executor,
            BackgroundTaskRunnerInput::new("worker-a", ts(1)).expect("input"),
        )
        .await
        .expect("runner")
        .expect("outcome");

    assert_eq!(outcome.task_id, task_id);
    assert_eq!(outcome.final_state, TaskState::Succeeded);
    assert!(outcome.event_count > 0);
    assert_eq!(outcome.trace_id, Some(task_id.to_string()));
    assert_eq!(outcome.summary, "background done");
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Succeeded
    );
}

#[tokio::test]
async fn background_turn_runner_marks_failed_turn_and_redacts_diagnostics() {
    let mut queue = queue();
    let task = task_run("fail safely");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let mut executor = RecordingExecutor::failure("model leaked sk-secret-token");

    let outcome = BackgroundTaskRunner
        .run_one(
            &mut queue,
            &mut executor,
            BackgroundTaskRunnerInput::new("worker-a", ts(1)).expect("input"),
        )
        .await
        .expect("runner")
        .expect("outcome");

    assert_eq!(outcome.task_id, task_id);
    assert_eq!(outcome.final_state, TaskState::Failed);
    assert_eq!(outcome.event_count, 0);
    assert!(outcome.summary.contains("[redacted]"));
    assert!(!outcome.summary.contains("sk-secret-token"));
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Failed
    );
}

struct RecordingExecutor {
    result: Result<BackgroundTaskTurnResult, BackgroundTaskTurnError>,
    executed_task_count: usize,
}

impl RecordingExecutor {
    fn success(summary: &str) -> Self {
        Self {
            result: Ok(BackgroundTaskTurnResult {
                event_count: 1,
                trace_id: Some("trace".to_string()),
                summary: summary.to_string(),
                approval_id: None,
            }),
            executed_task_count: 0,
        }
    }

    fn failure(message: &str) -> Self {
        Self {
            result: Err(BackgroundTaskTurnError::new(message)),
            executed_task_count: 0,
        }
    }
}

#[async_trait::async_trait]
impl BackgroundTaskTurnExecutor for RecordingExecutor {
    async fn execute_task_turn(
        &mut self,
        _task: &TaskRun,
    ) -> Result<BackgroundTaskTurnResult, BackgroundTaskTurnError> {
        self.executed_task_count += 1;
        self.result.clone()
    }
}
