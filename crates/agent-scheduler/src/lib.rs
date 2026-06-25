#![forbid(unsafe_code)]
//! Scheduler contracts for durable tasks and background work.

pub mod error;
pub mod task;

pub use error::SchedulerError;
pub use task::{TaskKind, TaskPriority, TaskRun, TaskTransition, transition_task_state};
