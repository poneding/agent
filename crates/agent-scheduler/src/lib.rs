#![forbid(unsafe_code)]
//! Scheduler contracts for durable tasks and background work.

pub mod command;
pub mod error;
pub mod lease;
pub mod notification;
pub mod queue;
pub mod quota;
pub mod recovery;
pub mod retry;
pub mod task;

pub use command::{
    BackgroundAgentTask, InMemoryTaskCommandService, TaskCommand, TaskCommandResponse,
    TaskCommandService, TaskCreateRequest, TaskSchedule, TaskScheduleUpdate, redact_task_text,
};
pub use error::SchedulerError;
pub use lease::{LeaseStatus, TaskLease, TaskLeaseId};
pub use notification::{
    InMemoryTaskNotificationSink, TaskNotification, TaskNotificationKind, TaskNotificationSink,
};
pub use queue::{InMemoryTaskQueue, TaskQueue};
pub use quota::{
    QuotaDecision, QuotaPolicy, QuotaRejection, QuotaSnapshot, TaskUsageQuotaPolicy,
    TaskUsageSnapshot, evaluate_task_quota, evaluate_task_usage_quota,
};
pub use recovery::{StaleLeaseDisposition, StaleLeaseRecoveryPolicy, TaskRecoveryReport};
pub use retry::{RetryAttemptSummary, RetryDecision, RetryPolicy};
pub use task::{TaskKind, TaskPriority, TaskRun, TaskTransition, transition_task_state};
