use ruse_types::{ApprovalRequestId, TaskId, TaskState, TodoStatus};
use ruse_ui_bridge::{
    UiEventPayload, UiJsonFrame, UiMemorySnapshot, UiPlanTodoSnapshot, UiTaskReviewSummary,
    UiTaskSummary, UiTeamWorkflowSnapshot,
};

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
    pub plan: Option<UiPlanTodoSnapshot>,
    pub reviews: Vec<UiTaskReviewSummary>,
    pub team_workflows: Vec<UiTeamWorkflowSnapshot>,
    pub memory: Option<UiMemorySnapshot>,
    pub notifications: Vec<TaskPanelNotification>,
    limit: usize,
}

impl TaskPanelState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            tasks: Vec::new(),
            plan: None,
            reviews: Vec::new(),
            team_workflows: Vec::new(),
            memory: None,
            notifications: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::Admin(admin) if admin.kind == "task_snapshot" => {
                if let Ok(snapshot) = admin.decode_as::<ruse_ui_bridge::UiTaskSnapshot>() {
                    self.tasks = snapshot.tasks.clone();
                }
            }
            UiJsonFrame::PlanTodoSnapshot(snapshot) => {
                self.plan = Some(snapshot.clone());
            }
            UiJsonFrame::Admin(admin) if admin.kind == "task_review_snapshot" => {
                if let Ok(snapshot) = admin.decode_as::<ruse_ui_bridge::UiTaskReviewSnapshot>() {
                    self.reviews = snapshot.reviews.clone();
                }
            }
            UiJsonFrame::Admin(admin) if admin.kind == "team_workflow_snapshot" => {
                if let Ok(snapshot) = admin.decode_as::<ruse_ui_bridge::UiTeamWorkflowSnapshot>() {
                    self.team_workflows = vec![snapshot];
                }
            }
            UiJsonFrame::Admin(admin) if admin.kind == "memory_snapshot" => {
                if let Ok(snapshot) = admin.decode_as::<ruse_ui_bridge::UiMemorySnapshot>() {
                    self.memory = Some(snapshot);
                }
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
        if self.tasks.is_empty()
            && self.plan.is_none()
            && self.reviews.is_empty()
            && self.team_workflows.is_empty()
            && self.memory.is_none()
            && self.notifications.is_empty()
        {
            return "Tasks: empty".to_string();
        }

        let mut lines = Vec::new();
        if let Some(plan) = &self.plan {
            lines.push(format!(
                "Plan: revision={} total={} pending={} in_progress={} completed={} cancelled={}",
                plan.snapshot.revision,
                plan.progress.total,
                plan.progress.pending,
                plan.progress.in_progress,
                plan.progress.completed,
                plan.progress.cancelled
            ));
            if let Some(active) = plan
                .snapshot
                .items
                .iter()
                .find(|item| item.status == TodoStatus::InProgress)
            {
                lines.push(format!(
                    "active todo {} priority={} content={}",
                    active.item_id,
                    format!("{:?}", active.priority).to_ascii_lowercase(),
                    active.content
                ));
            } else {
                lines.push("active todo none".to_string());
            }
            for diagnostic in &plan.snapshot.diagnostics {
                lines.push(format!(
                    "plan diagnostic {} {}",
                    diagnostic.code, diagnostic.message
                ));
            }
            for error in &plan.errors {
                lines.push(format!("plan error {} {}", error.code, error.message));
            }
        }
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
            if let Some(progress) = task.plan_progress {
                line.push_str(&format!(
                    " plan={}/{}/{}",
                    progress.completed, progress.in_progress, progress.total
                ));
            }
            if let Some(active_todo) = &task.active_todo {
                line.push_str(&format!(" active_todo={}", active_todo.content));
            }
            if task.waiting_approval {
                line.push_str(" waiting_approval=yes");
            }
            if let Some(notification) = &task.latest_notification {
                line.push_str(&format!(" notification={notification}"));
            }
            if task.artifact_count > 0 {
                line.push_str(&format!(" artifacts={}", task.artifact_count));
            }
            if !task.recovery_conflicts.is_empty() {
                line.push_str(&format!(" conflicts={}", task.recovery_conflicts.len()));
                for conflict in &task.recovery_conflicts {
                    line.push_str(&format!(
                        " conflict={} kind={} next_action={}",
                        conflict.path,
                        conflict_kind_label(conflict.kind),
                        conflict_next_action_label(conflict.next_action)
                    ));
                }
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
        for team in &self.team_workflows {
            lines.push(format!(
                "Team workflow: operation={} status={:?} readiness={:?} dry_run={} pending_approvals={} conflicts={} diagnostics={} errors={}",
                team.operation,
                team.snapshot.status,
                team.snapshot.readiness,
                team.dry_run,
                team.snapshot.pending_approval_count,
                team.snapshot.conflicts.len(),
                team.snapshot.diagnostics.len(),
                team.errors.len()
            ));
            lines.push(format!(
                "team capability grants={} policy_blocked={}",
                team.snapshot.capability_ceiling.grant_count,
                team.snapshot.capability_ceiling.policy_blocked
            ));
            lines.push(format!(
                "team blackboard entries={} approvals={} request_changes={} blocks={}",
                team.snapshot.blackboard.entry_count,
                team.snapshot.blackboard.approve_count,
                team.snapshot.blackboard.request_changes_count,
                team.snapshot.blackboard.block_count
            ));
            lines.push(format!(
                "team merge_review required={} approving={} blocking={} artifacts={} approved={}",
                team.snapshot.merge_review.required_review_count,
                team.snapshot.merge_review.approving_review_count,
                team.snapshot.merge_review.blocking_review_count,
                team.snapshot.merge_review.artifact_count,
                team.snapshot.merge_review.approved
            ));
            for handoff in &team.snapshot.handoffs {
                lines.push(format!(
                    "team handoff {} role={:?} status={:?} objective={}",
                    handoff.subagent_id, handoff.role, handoff.status, handoff.objective
                ));
            }
            for conflict in &team.snapshot.conflicts {
                lines.push(format!(
                    "team conflict {} kind={} next_action={}",
                    conflict.path,
                    conflict_kind_label(conflict.kind),
                    conflict_next_action_label(conflict.next_action)
                ));
            }
            for diagnostic in &team.snapshot.diagnostics {
                lines.push(format!(
                    "team diagnostic {:?} {} {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                ));
            }
        }
        if let Some(memory) = &self.memory {
            lines.push(format!(
                "Memory: scope={} enabled={} layers={} records={} proposals={} selected={} rejected={} tombstones={} diagnostics={}",
                memory.scope_label,
                memory.enabled,
                memory.layer_count,
                memory.record_count,
                memory.proposal_count,
                memory.selected_count,
                memory.rejected_count,
                memory.tombstone_count,
                memory.diagnostics.len()
            ));
            lines.push(format!(
                "memory retention export_sensitive={} tombstones_retained={} sensitive_review={}",
                memory.retention.export_includes_sensitive,
                memory.retention.tombstones_retained,
                memory.retention.sensitive_memory_requires_review
            ));
            for record in &memory.records {
                lines.push(format!(
                    "memory record {} scope={} sensitivity={} confidence={} preview={}",
                    record.id,
                    record.scope,
                    record.sensitivity,
                    record.confidence,
                    record.text_preview.as_deref().unwrap_or("[hidden]")
                ));
            }
            for proposal in &memory.proposals {
                lines.push(format!(
                    "memory proposal {} status={} sensitivity={} review_required={} redacted={} summary={}",
                    proposal.proposal_id,
                    proposal.status,
                    proposal.sensitivity,
                    proposal.requires_explicit_approval,
                    proposal.content_redacted,
                    proposal.summary
                ));
            }
            if let Some(retrieval) = &memory.retrieval {
                lines.push(format!(
                    "memory retrieval query={} selected={} rejected={} candidates={}",
                    retrieval.query_hash,
                    retrieval.selected_count,
                    retrieval.rejected_count,
                    retrieval.candidates.len()
                ));
                for candidate in &retrieval.candidates {
                    lines.push(format!(
                        "memory candidate {} selected={} rank={} score={} reasons={} preview={}",
                        candidate.memory_id,
                        candidate.selected,
                        candidate.rank,
                        candidate.combined_score,
                        candidate.reasons.join(","),
                        candidate.preview.as_deref().unwrap_or("[hidden]")
                    ));
                }
            }
            if let Some(influence) = &memory.influence {
                lines.push(format!(
                    "memory influence query={} selected={} rejected={} entries={}",
                    influence.query_hash,
                    influence.selected_count,
                    influence.rejected_count,
                    influence.entries.len()
                ));
            }
            for diagnostic in &memory.diagnostics {
                lines.push(format!(
                    "memory diagnostic {} {} {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                ));
            }
        }
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

fn conflict_kind_label(kind: ruse_types::RecoveryConflictKind) -> &'static str {
    match kind {
        ruse_types::RecoveryConflictKind::BothModified => "both_modified",
        ruse_types::RecoveryConflictKind::DeleteModify => "delete_modify",
        ruse_types::RecoveryConflictKind::AddAddDifferentContent => "add_add_different_content",
    }
}

fn conflict_next_action_label(action: ruse_types::RecoveryConflictNextAction) -> &'static str {
    match action {
        ruse_types::RecoveryConflictNextAction::InspectDiff => "inspect_diff",
        ruse_types::RecoveryConflictNextAction::ChooseSide => "choose_side",
        ruse_types::RecoveryConflictNextAction::ForkWork => "fork_work",
        ruse_types::RecoveryConflictNextAction::ManualResolution => "manual_resolution",
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
