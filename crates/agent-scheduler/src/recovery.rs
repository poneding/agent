use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleLeaseDisposition {
    Requeue,
    Fail,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StaleLeaseRecoveryPolicy {
    pub disposition: StaleLeaseDisposition,
}

impl StaleLeaseRecoveryPolicy {
    #[must_use]
    pub const fn requeue() -> Self {
        Self {
            disposition: StaleLeaseDisposition::Requeue,
        }
    }

    #[must_use]
    pub const fn fail() -> Self {
        Self {
            disposition: StaleLeaseDisposition::Fail,
        }
    }
}

impl Default for StaleLeaseRecoveryPolicy {
    fn default() -> Self {
        Self::requeue()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskRecoveryReport {
    pub expired_leases: usize,
    pub requeued_tasks: usize,
    pub failed_tasks: usize,
}

impl TaskRecoveryReport {
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.expired_leases == 0 && self.requeued_tasks == 0 && self.failed_tasks == 0
    }
}
