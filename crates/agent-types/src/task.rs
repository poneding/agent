use serde::{Deserialize, Serialize};

use crate::ids::TaskId;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskNotificationRecorded {
    pub task_id: TaskId,
    pub kind: String,
    pub summary: String,
}
