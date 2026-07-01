use std::collections::BTreeSet;

use agent_scheduler::redact_task_text;
use agent_types::{
    PlanTodoEvidence, PlanTodoItem, PlanTodoItemId, PlanTodoProgress, PlanTodoSnapshot, TodoStatus,
};
use thiserror::Error;

const TODO_CONTENT_LIMIT: usize = 240;
const TODO_REASON_LIMIT: usize = 240;
const TODO_EVIDENCE_LIMIT: usize = 160;
const TODO_DIAGNOSTIC_LIMIT: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanTodoUpdate {
    pub expected_revision: u64,
    pub reason: String,
    pub operation: PlanTodoOperation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanTodoOperation {
    ReplaceAll {
        items: Vec<PlanTodoItem>,
    },
    UpdateStatus {
        item_id: PlanTodoItemId,
        status: TodoStatus,
    },
    AppendEvidence {
        item_id: PlanTodoItemId,
        evidence: PlanTodoEvidence,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanTodoUpdateOutcome {
    pub snapshot: PlanTodoSnapshot,
    pub previous_revision: u64,
    pub progress: PlanTodoProgress,
    pub reason: String,
}

pub fn validate_plan_todo_snapshot(
    snapshot: &PlanTodoSnapshot,
) -> Result<PlanTodoProgress, PlanTodoError> {
    let mut item_ids = BTreeSet::new();
    let mut in_progress = 0_u32;

    for item in &snapshot.items {
        validate_item(item)?;
        if !item_ids.insert(item.item_id.clone()) {
            return Err(PlanTodoError::Validation(format!(
                "duplicate todo item id: {}",
                item.item_id
            )));
        }
        if item.status == TodoStatus::InProgress {
            in_progress = in_progress.saturating_add(1);
        }
    }

    if in_progress > 1 {
        return Err(PlanTodoError::Validation(
            "plan/todo snapshot cannot have more than one in_progress item".to_string(),
        ));
    }

    for diagnostic in &snapshot.diagnostics {
        validate_required("diagnostic code", &diagnostic.code)?;
        validate_required("diagnostic message", &diagnostic.message)?;
    }

    Ok(progress_for_items(&snapshot.items))
}

pub fn normalize_plan_todo_snapshot(
    snapshot: PlanTodoSnapshot,
) -> Result<(PlanTodoSnapshot, PlanTodoProgress), PlanTodoError> {
    let mut next = snapshot;
    for item in &mut next.items {
        normalize_item(item)?;
    }
    for diagnostic in &mut next.diagnostics {
        diagnostic.code =
            normalize_required("diagnostic code", &diagnostic.code, TODO_DIAGNOSTIC_LIMIT)?;
        diagnostic.message = normalize_required(
            "diagnostic message",
            &diagnostic.message,
            TODO_DIAGNOSTIC_LIMIT,
        )?;
    }
    let progress = validate_plan_todo_snapshot(&next)?;
    Ok((next, progress))
}

pub fn apply_plan_todo_update(
    snapshot: &PlanTodoSnapshot,
    update: PlanTodoUpdate,
) -> Result<PlanTodoUpdateOutcome, PlanTodoError> {
    if update.expected_revision != snapshot.revision {
        return Err(PlanTodoError::StaleRevision {
            expected: update.expected_revision,
            actual: snapshot.revision,
        });
    }
    let reason = normalize_required("update reason", &update.reason, TODO_REASON_LIMIT)?;

    let mut next = snapshot.clone();
    let updated_at = next.updated_at;
    match update.operation {
        PlanTodoOperation::ReplaceAll { items } => {
            next.items = items;
        }
        PlanTodoOperation::UpdateStatus { item_id, status } => {
            let item = next
                .items
                .iter_mut()
                .find(|item| item.item_id == item_id)
                .ok_or_else(|| PlanTodoError::MissingItem(item_id.to_string()))?;
            if matches!(item.status, TodoStatus::Completed | TodoStatus::Cancelled)
                && item.status != status
            {
                return Err(PlanTodoError::TerminalItemStatus {
                    item_id: item.item_id.to_string(),
                    status: item.status,
                });
            }
            item.status = status;
            item.updated_at = updated_at;
        }
        PlanTodoOperation::AppendEvidence { item_id, evidence } => {
            let item = next
                .items
                .iter_mut()
                .find(|item| item.item_id == item_id)
                .ok_or_else(|| PlanTodoError::MissingItem(item_id.to_string()))?;
            let evidence = normalized_evidence(evidence)?;
            item.evidence.push(evidence);
            item.updated_at = updated_at;
        }
    }

    next.revision = snapshot.revision.saturating_add(1);
    let (next, progress) = normalize_plan_todo_snapshot(next)?;

    Ok(PlanTodoUpdateOutcome {
        snapshot: next,
        previous_revision: snapshot.revision,
        progress,
        reason,
    })
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PlanTodoError {
    #[error("invalid plan/todo state: {0}")]
    Validation(String),
    #[error("stale plan/todo revision: expected {expected}, actual {actual}")]
    StaleRevision { expected: u64, actual: u64 },
    #[error("todo item not found: {0}")]
    MissingItem(String),
    #[error("terminal todo item {item_id} cannot change status from {status:?}")]
    TerminalItemStatus { item_id: String, status: TodoStatus },
}

fn validate_item(item: &PlanTodoItem) -> Result<(), PlanTodoError> {
    validate_required("todo item id", item.item_id.as_str())?;
    validate_required("todo content", &item.content)?;
    for evidence in &item.evidence {
        validate_required("todo evidence reference", &evidence.reference)?;
        validate_required("todo evidence summary", &evidence.summary)?;
    }
    Ok(())
}

fn normalize_item(item: &mut PlanTodoItem) -> Result<(), PlanTodoError> {
    item.content = normalize_required("todo content", &item.content, TODO_CONTENT_LIMIT)?;
    let mut evidence = Vec::with_capacity(item.evidence.len());
    for item_evidence in item.evidence.drain(..) {
        evidence.push(normalized_evidence(item_evidence)?);
    }
    item.evidence = evidence;
    Ok(())
}

fn normalized_evidence(mut evidence: PlanTodoEvidence) -> Result<PlanTodoEvidence, PlanTodoError> {
    evidence.reference = normalize_required(
        "todo evidence reference",
        &evidence.reference,
        TODO_EVIDENCE_LIMIT,
    )?;
    evidence.summary = normalize_required(
        "todo evidence summary",
        &evidence.summary,
        TODO_EVIDENCE_LIMIT,
    )?;
    Ok(evidence)
}

fn progress_for_items(items: &[PlanTodoItem]) -> PlanTodoProgress {
    let mut progress = PlanTodoProgress {
        total: items.len().try_into().unwrap_or(u32::MAX),
        ..PlanTodoProgress::default()
    };
    for item in items {
        match item.status {
            TodoStatus::Pending => progress.pending = progress.pending.saturating_add(1),
            TodoStatus::InProgress => {
                progress.in_progress = progress.in_progress.saturating_add(1);
            }
            TodoStatus::Completed => progress.completed = progress.completed.saturating_add(1),
            TodoStatus::Cancelled => progress.cancelled = progress.cancelled.saturating_add(1),
        }
    }
    progress
}

fn validate_required(label: &str, value: &str) -> Result<(), PlanTodoError> {
    if value.trim().is_empty() {
        return Err(PlanTodoError::Validation(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(())
}

fn normalize_required(label: &str, value: &str, limit: usize) -> Result<String, PlanTodoError> {
    validate_required(label, value)?;
    Ok(bounded_redacted(value.trim(), limit))
}

fn bounded_redacted(value: &str, limit: usize) -> String {
    let redacted = redact_task_text(value);
    if redacted.chars().count() > limit {
        redacted.chars().take(limit).collect()
    } else {
        redacted
    }
}
