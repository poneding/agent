use serde::{Deserialize, Serialize};

use ruse_types::{TaskId, TaskState, Timestamp};

use crate::{SchedulerError, TaskRun, redact_task_text};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl RetryPolicy {
    pub(crate) fn validate(self) -> Result<Self, SchedulerError> {
        if self.max_attempts == 0 {
            return Err(SchedulerError::InvalidTask(
                "retry max_attempts must be at least 1".to_string(),
            ));
        }
        if self.base_delay_ms == 0 {
            return Err(SchedulerError::InvalidTask(
                "retry base_delay_ms must be at least 1".to_string(),
            ));
        }
        if self.max_delay_ms < self.base_delay_ms {
            return Err(SchedulerError::InvalidTask(
                "retry max_delay_ms must be greater than or equal to base_delay_ms".to_string(),
            ));
        }
        Ok(self)
    }

    pub(crate) fn delay_ms_for_attempt(self, attempt: u32) -> u64 {
        let exponent = attempt.saturating_sub(1).min(63);
        let multiplier = 1_u64.checked_shl(exponent).unwrap_or(u64::MAX);
        self.base_delay_ms
            .saturating_mul(multiplier)
            .min(self.max_delay_ms)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 1,
            base_delay_ms: 1_000,
            max_delay_ms: 60_000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryDecision {
    RetryAt {
        task: TaskRun,
        run_at: Timestamp,
        error: String,
    },
    Exhausted {
        task: TaskRun,
        error: String,
    },
}

impl RetryDecision {
    #[must_use]
    pub fn summary(&self) -> RetryAttemptSummary {
        match self {
            Self::RetryAt {
                task,
                run_at,
                error,
            } => RetryAttemptSummary {
                task_id: task.task_id,
                state: task.state,
                attempt: task.attempt,
                max_attempts: task.max_attempts,
                next_run_at: Some(*run_at),
                error: Some(redact_task_text(error)),
            },
            Self::Exhausted { task, error } => RetryAttemptSummary {
                task_id: task.task_id,
                state: task.state,
                attempt: task.attempt,
                max_attempts: task.max_attempts,
                next_run_at: None,
                error: Some(redact_task_text(error)),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetryAttemptSummary {
    pub task_id: TaskId,
    pub state: TaskState,
    pub attempt: u32,
    pub max_attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
