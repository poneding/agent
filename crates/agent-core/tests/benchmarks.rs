use agent_core::{
    BenchmarkEnvironment, BenchmarkError, BenchmarkRunner, BenchmarkScenario, run_smoke_benchmarks,
};
use serde_json::json;

#[test]
fn benchmarks_scenario_serializes_stable_shape() {
    let scenario = BenchmarkScenario::new("event_append", 3, 2).expect("valid scenario");

    assert_eq!(
        serde_json::to_value(scenario).expect("serialize scenario"),
        json!({
            "name": "event_append",
            "iterations": 3,
            "operation_count": 2
        })
    );
}

#[test]
fn benchmarks_runner_rejects_invalid_scenario() {
    let runner = BenchmarkRunner::new(BenchmarkEnvironment::deterministic("test-os", "test-arch"));

    let empty_name = runner
        .run(
            BenchmarkScenario {
                name: " ".to_string(),
                iterations: 1,
                operation_count: 1,
            },
            || Ok(()),
        )
        .expect_err("empty name rejected");
    let zero_iterations = runner
        .run(
            BenchmarkScenario {
                name: "event_append".to_string(),
                iterations: 0,
                operation_count: 1,
            },
            || Ok(()),
        )
        .expect_err("zero iterations rejected");

    assert!(matches!(empty_name, BenchmarkError::EmptyScenarioName));
    assert!(matches!(zero_iterations, BenchmarkError::ZeroIterations));
}

#[test]
fn benchmarks_runner_records_operation_counts() {
    let runner = BenchmarkRunner::new(BenchmarkEnvironment::deterministic("test-os", "test-arch"));
    let mut calls = 0;

    let result = runner
        .run(
            BenchmarkScenario::new("tool_dispatch", 3, 4).expect("valid scenario"),
            || {
                calls += 1;
                Ok(())
            },
        )
        .expect("run benchmark");

    assert_eq!(calls, 3);
    assert_eq!(result.scenario_name, "tool_dispatch");
    assert_eq!(result.iterations, 3);
    assert_eq!(result.operation_count, 12);
}

#[test]
fn benchmarks_result_includes_environment_summary() {
    let environment = BenchmarkEnvironment::deterministic("test-os", "test-arch");
    let runner = BenchmarkRunner::new(environment.clone());

    let result = runner
        .run(
            BenchmarkScenario::new("context_compaction", 1, 5).expect("valid scenario"),
            || Ok(()),
        )
        .expect("run benchmark");

    assert_eq!(result.environment, environment);
    assert_eq!(result.environment.os, "test-os");
    assert_eq!(result.environment.arch, "test-arch");
}

#[test]
fn benchmarks_smoke_scenarios_execute_without_external_services() {
    let results = run_smoke_benchmarks().expect("run smoke benchmarks");
    let names = results
        .iter()
        .map(|result| result.scenario_name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "turn_replay",
            "tool_dispatch",
            "event_append",
            "context_compaction"
        ]
    );
    assert!(results.iter().all(|result| result.operation_count > 0));
}
