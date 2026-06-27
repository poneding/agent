use serde::{Deserialize, Serialize};

use crate::ids::{ApprovalRequestId, TaskId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskNotificationRecorded {
    pub task_id: TaskId,
    pub kind: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<ApprovalRequestId>,
}
