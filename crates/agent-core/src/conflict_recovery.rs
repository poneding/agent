use agent_types::{
    RecoveryConflictKind, RecoveryConflictNextAction, RecoveryConflictSide,
    RecoveryConflictSummary, RecoveryDiagnostic, RecoveryDiagnosticSeverity, RecoveryReadiness,
    RecoverySnapshot,
};
use agent_worktree::{ChangeKind, ConflictKind, ConflictReport, FileChange};

const MAX_CONFLICT_TEXT_CHARS: usize = 240;

#[derive(Clone, Debug, Default)]
pub struct ConflictRecoveryMapper;

impl ConflictRecoveryMapper {
    #[must_use]
    pub fn map_report(&self, report: &ConflictReport) -> Vec<RecoveryConflictSummary> {
        let mut conflicts = report
            .conflicts
            .iter()
            .map(|conflict| RecoveryConflictSummary {
                path: bound_text(&conflict.path),
                kind: recovery_conflict_kind(conflict.kind),
                lead_change: recovery_side("lead", &conflict.lead_change),
                incoming_change: recovery_side("incoming", &conflict.subagent_change),
                next_action: next_action(conflict.kind),
                diagnostics: Vec::new(),
            })
            .collect::<Vec<_>>();
        conflicts.sort_by(|left, right| left.path.cmp(&right.path));
        conflicts
    }

    #[must_use]
    pub fn apply_to_snapshot(
        &self,
        mut snapshot: RecoverySnapshot,
        conflicts: Vec<RecoveryConflictSummary>,
    ) -> RecoverySnapshot {
        if conflicts.is_empty() {
            return snapshot;
        }

        snapshot.readiness = RecoveryReadiness::HasConflict;
        snapshot
            .diagnostics
            .push(conflict_diagnostic(conflicts.len()));
        snapshot.conflicts = conflicts;
        snapshot
    }
}

#[must_use]
pub fn conflict_diagnostic(conflict_count: usize) -> RecoveryDiagnostic {
    RecoveryDiagnostic {
        severity: RecoveryDiagnosticSeverity::Error,
        code: "recovery.conflict.detected".to_string(),
        message: format!(
            "{conflict_count} worktree conflict(s) require inspection before automatic resume"
        ),
        source: Some("agent-worktree".to_string()),
        truncated: false,
    }
}

fn recovery_conflict_kind(kind: ConflictKind) -> RecoveryConflictKind {
    match kind {
        ConflictKind::BothModified => RecoveryConflictKind::BothModified,
        ConflictKind::DeleteModify => RecoveryConflictKind::DeleteModify,
        ConflictKind::AddAddDifferentContent => RecoveryConflictKind::AddAddDifferentContent,
    }
}

fn recovery_side(source: &str, change: &FileChange) -> RecoveryConflictSide {
    RecoveryConflictSide {
        source: source.to_string(),
        change_kind: change_kind_label(change.kind).to_string(),
        content_hash: change.content_hash.as_ref().map(bound_text),
    }
}

fn next_action(kind: ConflictKind) -> RecoveryConflictNextAction {
    match kind {
        ConflictKind::BothModified => RecoveryConflictNextAction::InspectDiff,
        ConflictKind::DeleteModify => RecoveryConflictNextAction::ChooseSide,
        ConflictKind::AddAddDifferentContent => RecoveryConflictNextAction::ManualResolution,
    }
}

fn change_kind_label(kind: ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Added => "added",
        ChangeKind::Modified => "modified",
        ChangeKind::Deleted => "deleted",
    }
}

fn bound_text(value: impl AsRef<str>) -> String {
    let value = redact_secret_like(value.as_ref());
    if value.chars().count() <= MAX_CONFLICT_TEXT_CHARS {
        return value;
    }
    let mut output = value
        .chars()
        .take(MAX_CONFLICT_TEXT_CHARS)
        .collect::<String>();
    output.push_str("...");
    output
}

fn redact_secret_like(value: &str) -> String {
    value
        .split('/')
        .map(|segment| {
            segment
                .split_whitespace()
                .map(|part| {
                    let lower = part.to_ascii_lowercase();
                    if lower.contains("secret")
                        || lower.contains("token")
                        || lower.contains("api_key")
                        || lower.contains("apikey")
                        || lower.contains("password")
                        || lower.starts_with("sk-")
                    {
                        "[redacted]"
                    } else {
                        part
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("/")
}
