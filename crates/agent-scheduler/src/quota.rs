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

impl QuotaDecision {
    #[must_use]
    pub const fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    #[must_use]
    pub fn into_rejection(self) -> Option<QuotaRejection> {
        match self {
            Self::Allow => None,
            Self::Reject(rejection) => Some(rejection),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaRejection {
    pub code: String,
    pub message: String,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskUsageSnapshot {
    pub model_calls: u32,
    pub tool_calls: u32,
    pub output_bytes: u64,
    pub elapsed_ms: u64,
    pub worktree_count: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskUsageQuotaPolicy {
    pub max_model_calls: u32,
    pub max_tool_calls: u32,
    pub max_output_bytes: u64,
    pub max_elapsed_ms: u64,
    pub max_worktree_count: u32,
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

#[must_use]
pub fn evaluate_task_usage_quota(
    policy: &TaskUsageQuotaPolicy,
    snapshot: &TaskUsageSnapshot,
) -> QuotaDecision {
    if snapshot.model_calls >= policy.max_model_calls {
        return reject(
            "model_call_limit",
            "background task model call quota has been reached",
        );
    }
    if snapshot.tool_calls >= policy.max_tool_calls {
        return reject(
            "tool_call_limit",
            "background task tool call quota has been reached",
        );
    }
    if snapshot.output_bytes >= policy.max_output_bytes {
        return reject(
            "output_byte_limit",
            "background task output byte quota has been reached",
        );
    }
    if snapshot.elapsed_ms >= policy.max_elapsed_ms {
        return reject(
            "wall_time_limit",
            "background task wall time quota has been reached",
        );
    }
    if snapshot.worktree_count >= policy.max_worktree_count {
        return reject(
            "worktree_limit",
            "background task worktree quota has been reached",
        );
    }
    QuotaDecision::Allow
}

fn reject(code: &str, message: &str) -> QuotaDecision {
    QuotaDecision::Reject(QuotaRejection {
        code: code.to_string(),
        message: message.to_string(),
    })
}
