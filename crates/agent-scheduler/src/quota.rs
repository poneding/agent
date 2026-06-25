use serde::{Deserialize, Serialize};

use crate::{TaskKind, TaskRun};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub max_system_running: u32,
    pub max_workspace_running: u32,
    pub max_background_running: u32,
    pub max_cron_running: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaSnapshot {
    pub system_running: u32,
    pub workspace_running: u32,
    pub background_running: u32,
    pub cron_running: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaDecision {
    Allow,
    Reject(QuotaRejection),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaRejection {
    pub code: String,
    pub message: String,
}

#[must_use]
pub fn evaluate_task_quota(
    policy: &QuotaPolicy,
    snapshot: &QuotaSnapshot,
    candidate: &TaskRun,
) -> QuotaDecision {
    if snapshot.system_running >= policy.max_system_running {
        return reject(
            "system_running_limit",
            "system running task quota has been reached",
        );
    }
    if snapshot.workspace_running >= policy.max_workspace_running {
        return reject(
            "workspace_running_limit",
            "workspace running task quota has been reached",
        );
    }

    match candidate.kind {
        TaskKind::Background if snapshot.background_running >= policy.max_background_running => {
            reject(
                "background_running_limit",
                "background running task quota has been reached",
            )
        }
        TaskKind::Cron if snapshot.cron_running >= policy.max_cron_running => reject(
            "cron_running_limit",
            "cron running task quota has been reached",
        ),
        TaskKind::OneShot | TaskKind::Background | TaskKind::Cron => QuotaDecision::Allow,
    }
}

fn reject(code: &str, message: &str) -> QuotaDecision {
    QuotaDecision::Reject(QuotaRejection {
        code: code.to_string(),
        message: message.to_string(),
    })
}
