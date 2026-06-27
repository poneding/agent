use agent_types::{ApprovalRequestId, TaskId};
use agent_ui_bridge::{UiEventPayload, UiJsonFrame, UiTaskSummary};

const DEFAULT_TASK_NOTIFICATION_LIMIT: usize = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPanelNotification {
    pub task_id: TaskId,
    pub kind: String,
    pub summary: String,
    pub approval_id: Option<ApprovalRequestId>,
}

impl TaskPanelNotification {
    #[must_use]
    pub fn new(
        task_id: TaskId,
        kind: impl Into<String>,
        summary: impl Into<String>,
        approval_id: Option<ApprovalRequestId>,
    ) -> Self {
        Self {
            task_id,
            kind: kind.into(),
            summary: summary.into(),
            approval_id,
        }
    }

    #[must_use]
    pub fn render_line(&self) -> String {
        if let Some(approval_id) = self.approval_id {
            format!(
                "{} {} [{}]: {}",
                self.task_id, self.kind, approval_id, self.summary
            )
        } else {
            format!("{} {}: {}", self.task_id, self.kind, self.summary)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPanelState {
    pub tasks: Vec<UiTaskSummary>,
    pub notifications: Vec<TaskPanelNotification>,
    limit: usize,
}

impl TaskPanelState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            tasks: Vec::new(),
            notifications: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::TaskSnapshot(snapshot) => {
                self.tasks = snapshot.tasks.clone();
            }
            UiJsonFrame::Event(event) => {
                let UiEventPayload::TaskNotification(notification) = &event.payload else {
                    return;
                };

                self.push(TaskPanelNotification::new(
                    notification.task_id,
                    notification.kind.clone(),
                    notification.summary.clone(),
                    notification.approval_id,
                ));
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        if self.tasks.is_empty() && self.notifications.is_empty() {
            return "No task notifications".to_string();
        }

        let mut lines = self
            .tasks
            .iter()
            .map(|task| format!("{} {:?}: {}", task.task_id, task.state, task.prompt_summary))
            .collect::<Vec<_>>();
        lines.extend(
            self.notifications
                .iter()
                .map(TaskPanelNotification::render_line),
        );
        lines.join("\n")
    }

    fn push(&mut self, notification: TaskPanelNotification) {
        self.notifications.push(notification);
        let overflow = self.notifications.len().saturating_sub(self.limit);
        if overflow > 0 {
            self.notifications.drain(0..overflow);
        }
    }
}

impl Default for TaskPanelState {
    fn default() -> Self {
        Self::with_limit(DEFAULT_TASK_NOTIFICATION_LIMIT)
    }
}
