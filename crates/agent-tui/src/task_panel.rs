use agent_types::TaskId;
use agent_ui_bridge::{UiEventPayload, UiJsonFrame};

const DEFAULT_TASK_NOTIFICATION_LIMIT: usize = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPanelNotification {
    pub task_id: TaskId,
    pub kind: String,
    pub summary: String,
}

impl TaskPanelNotification {
    #[must_use]
    pub fn new(task_id: TaskId, kind: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            task_id,
            kind: kind.into(),
            summary: summary.into(),
        }
    }

    #[must_use]
    pub fn render_line(&self) -> String {
        format!("{} {}: {}", self.task_id, self.kind, self.summary)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPanelState {
    pub notifications: Vec<TaskPanelNotification>,
    limit: usize,
}

impl TaskPanelState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            notifications: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        let UiJsonFrame::Event(event) = frame else {
            return;
        };

        let UiEventPayload::TaskNotification {
            task_id,
            kind,
            summary,
        } = &event.payload
        else {
            return;
        };

        self.push(TaskPanelNotification::new(
            *task_id,
            kind.clone(),
            summary.clone(),
        ));
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        if self.notifications.is_empty() {
            return "No task notifications".to_string();
        }

        self.notifications
            .iter()
            .map(TaskPanelNotification::render_line)
            .collect::<Vec<_>>()
            .join("\n")
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
