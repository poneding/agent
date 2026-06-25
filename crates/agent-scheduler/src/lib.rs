#![forbid(unsafe_code)]
//! Scheduler contracts for durable tasks and background work.

pub mod error;
pub mod lease;
pub mod queue;
pub mod quota;
pub mod retry;
pub mod task;

pub use error::SchedulerError;
pub use lease::{LeaseStatus, TaskLease, TaskLeaseId};
pub use queue::{InMemoryTaskQueue, TaskQueue};
pub use quota::{QuotaDecision, QuotaPolicy, QuotaRejection, QuotaSnapshot, evaluate_task_quota};
pub use retry::{RetryDecision, RetryPolicy};
pub use task::{TaskKind, TaskPriority, TaskRun, TaskTransition, transition_task_state};
