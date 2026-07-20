use std::time::Instant;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkScenario {
    pub name: String,
    pub iterations: u32,
    pub operation_count: u64,
}

impl BenchmarkScenario {
    pub fn new(
        name: impl Into<String>,
        iterations: u32,
        operation_count: u64,
    ) -> Result<Self, BenchmarkError> {
        let scenario = Self {
            name: name.into(),
            iterations,
            operation_count,
        };
        scenario.validate()?;
        Ok(scenario)
    }

    fn validate(&self) -> Result<(), BenchmarkError> {
        if self.name.trim().is_empty() {
            return Err(BenchmarkError::EmptyScenarioName);
        }
        if self.iterations == 0 {
            return Err(BenchmarkError::ZeroIterations);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkEnvironment {
    pub os: String,
    pub arch: String,
    pub crate_version: String,
}

impl BenchmarkEnvironment {
    #[must_use]
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    #[must_use]
    pub fn deterministic(os: impl Into<String>, arch: impl Into<String>) -> Self {
        Self {
            os: os.into(),
            arch: arch.into(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub scenario_name: String,
    pub iterations: u32,
    pub operation_count: u64,
    pub elapsed_micros: u128,
    pub environment: BenchmarkEnvironment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkRunner {
    environment: BenchmarkEnvironment,
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new(BenchmarkEnvironment::current())
    }
}

impl BenchmarkRunner {
    #[must_use]
    pub fn new(environment: BenchmarkEnvironment) -> Self {
        Self { environment }
    }

    pub fn run<F>(
        &self,
        scenario: BenchmarkScenario,
        mut operation: F,
    ) -> Result<BenchmarkResult, BenchmarkError>
    where
        F: FnMut() -> Result<(), BenchmarkError>,
    {
        scenario.validate()?;
        let start = Instant::now();
        for _ in 0..scenario.iterations {
            operation()?;
        }
        let operation_count = scenario
            .operation_count
            .checked_mul(u64::from(scenario.iterations))
            .ok_or(BenchmarkError::OperationCountOverflow)?;

        Ok(BenchmarkResult {
            scenario_name: scenario.name,
            iterations: scenario.iterations,
            operation_count,
            elapsed_micros: start.elapsed().as_micros(),
            environment: self.environment.clone(),
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BenchmarkError {
    #[error("benchmark scenario name must not be empty")]
    EmptyScenarioName,
    #[error("benchmark scenario iterations must be greater than zero")]
    ZeroIterations,
    #[error("benchmark operation count overflowed")]
    OperationCountOverflow,
    #[error("benchmark operation failed: {0}")]
    OperationFailed(String),
}

#[must_use]
pub fn smoke_benchmark_scenarios() -> Vec<BenchmarkScenario> {
    vec![
        BenchmarkScenario::new("turn_replay", 2, 6).expect("valid smoke scenario"),
        BenchmarkScenario::new("tool_dispatch", 2, 5).expect("valid smoke scenario"),
        BenchmarkScenario::new("event_append", 2, 8).expect("valid smoke scenario"),
        BenchmarkScenario::new("context_compaction", 2, 4).expect("valid smoke scenario"),
    ]
}

pub fn run_smoke_benchmarks() -> Result<Vec<BenchmarkResult>, BenchmarkError> {
    let runner = BenchmarkRunner::default();
    smoke_benchmark_scenarios()
        .into_iter()
        .map(|scenario| {
            let operations = scenario.operation_count;
            runner.run(scenario, || {
                let mut checksum = 0_u64;
                for value in 0..operations {
                    checksum ^= value;
                }
                std::hint::black_box(checksum);
                Ok(())
            })
        })
        .collect()
}
