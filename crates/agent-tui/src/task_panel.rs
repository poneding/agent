use agent_types::{ApprovalRequestId, TaskId, TaskState};
use agent_ui_bridge::{UiEventPayload, UiJsonFrame, UiTaskReviewSummary, UiTaskSummary};

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
    pub reviews: Vec<UiTaskReviewSummary>,
    pub notifications: Vec<TaskPanelNotification>,
    limit: usize,
}

impl TaskPanelState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            tasks: Vec::new(),
            reviews: Vec::new(),
            notifications: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::TaskSnapshot(snapshot) => {
                self.tasks = snapshot.tasks.clone();
            }
            UiJsonFrame::TaskReviewSnapshot(snapshot) => {
                self.reviews = snapshot.reviews.clone();
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
        if self.tasks.is_empty() && self.reviews.is_empty() && self.notifications.is_empty() {
            return "Tasks: empty".to_string();
        }

        let mut lines = Vec::new();
        if !self.tasks.is_empty() {
            lines.push(format!("Tasks: {}", self.tasks.len()));
            for (state, count) in self.state_counts() {
                lines.push(format!("state {} count={count}", task_state_label(state)));
            }
        }
        lines.extend(self.tasks.iter().map(|task| {
            let mut line = format!(
                "{} state={} priority={} schedule={} prompt={}",
                task.task_id,
                task_state_label(task.state),
                format!("{:?}", task.priority).to_ascii_lowercase(),
                task.schedule.label,
                task.prompt_summary
            );
            if let Some(summary) = &task.latest_run_summary {
                line.push_str(&format!(" latest={summary}"));
            }
            line
        }));
        lines.extend(self.reviews.iter().map(|review| {
            let mut line = format!(
                "review {} state={}: {} artifacts={}",
                review.task_id,
                task_state_label(review.final_state),
                review.summary,
                review.artifact_count
            );
            if let Some(trace_id) = &review.trace_id {
                line.push_str(&format!(" trace={trace_id}"));
            }
            if let Some(checkpoint_id) = review.checkpoint_id {
                line.push_str(&format!(" checkpoint={checkpoint_id}"));
            }
            line
        }));
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

    fn state_counts(&self) -> Vec<(TaskState, u32)> {
        [
            TaskState::Created,
            TaskState::Queued,
            TaskState::Running,
            TaskState::WaitingApproval,
            TaskState::Sleeping,
            TaskState::Paused,
            TaskState::Succeeded,
            TaskState::Failed,
            TaskState::Cancelled,
            TaskState::Expired,
        ]
        .into_iter()
        .filter_map(|state| {
            let count = self.tasks.iter().filter(|task| task.state == state).count();
            (count > 0).then(|| (state, count.try_into().unwrap_or(u32::MAX)))
        })
        .collect()
    }
}

impl Default for TaskPanelState {
    fn default() -> Self {
        Self::with_limit(DEFAULT_TASK_NOTIFICATION_LIMIT)
    }
}

fn task_state_label(state: TaskState) -> &'static str {
    match state {
        TaskState::Created => "created",
        TaskState::Queued => "queued",
        TaskState::Running => "running",
        TaskState::WaitingApproval => "waiting_approval",
        TaskState::Sleeping => "sleeping",
        TaskState::Paused => "paused",
        TaskState::Succeeded => "succeeded",
        TaskState::Failed => "failed",
        TaskState::Cancelled => "cancelled",
        TaskState::Expired => "expired",
    }
}
