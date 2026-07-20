use ruse_types::{TaskId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskLeaseId(String);

impl TaskLeaseId {
    #[must_use]
    pub(crate) fn generated(sequence: u64) -> Self {
        Self(format!("lease-{sequence}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseStatus {
    Active,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskLease {
    pub id: TaskLeaseId,
    pub task_id: TaskId,
    pub worker_id: String,
    pub acquired_at: Timestamp,
    pub expires_at: Timestamp,
    pub status: LeaseStatus,
}
