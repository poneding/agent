use agent_types::{TaskId, TeamId};
use serde::{Deserialize, Serialize};

use crate::{
    HandoffStatus, TeamError, TeamWorkflowDiagnostic, TeamWorkflowReadiness, TeamWorkflowSnapshot,
    TeamWorktreeState,
};

const BLOCKER_MESSAGE_LIMIT: usize = 240;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamMergeBlockerKind {
    PolicyBlocked,
    Conflict,
    PendingApproval,
    StaleWorktree,
    WaitingOnSubagent,
    FailedHandoff,
    BlockingReview,
    MissingReview,
    MissingArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamMergeBlocker {
    pub kind: TeamMergeBlockerKind,
    pub count: u32,
    pub message: String,
}

impl TeamMergeBlocker {
    pub fn new(
        kind: TeamMergeBlockerKind,
        count: u32,
        message: impl Into<String>,
    ) -> Result<Self, TeamError> {
        if count == 0 {
            return Err(TeamError::InvalidWorkflow(
                "merge blocker count cannot be zero".to_string(),
            ));
        }
        Ok(Self {
            kind,
            count,
            message: merge_text("merge blocker message", message.into())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamMergeReadinessReport {
    pub team_id: TeamId,
    pub task_id: TaskId,
    pub readiness: TeamWorkflowReadiness,
    pub merge_ready: bool,
    pub pending_approval_count: u32,
    pub conflict_count: u32,
    pub stale_worktree_count: u32,
    pub required_review_count: u32,
    pub approving_review_count: u32,
    pub blocking_review_count: u32,
    pub artifact_count: u32,
    pub blockers: Vec<TeamMergeBlocker>,
    pub diagnostics: Vec<TeamWorkflowDiagnostic>,
}

impl TeamMergeReadinessReport {
    pub fn from_snapshot(snapshot: &TeamWorkflowSnapshot) -> Result<Self, TeamError> {
        let mut blockers = Vec::new();
        let stale_worktree_count = snapshot
            .worktrees
            .iter()
            .filter(|worktree| worktree.state == TeamWorktreeState::Stale)
            .count() as u32;
        let waiting_handoff_count = snapshot
            .handoffs
            .iter()
            .filter(|handoff| {
                matches!(
                    handoff.status,
                    HandoffStatus::Created | HandoffStatus::Accepted
                )
            })
            .count() as u32;
        let failed_handoff_count = snapshot
            .handoffs
            .iter()
            .filter(|handoff| {
                matches!(
                    handoff.status,
                    HandoffStatus::Failed | HandoffStatus::Cancelled
                )
            })
            .count() as u32;

        push_blocker(
            &mut blockers,
            snapshot.capability_ceiling.policy_blocked
                || snapshot.capability_ceiling.grant_count == 0,
            TeamMergeBlockerKind::PolicyBlocked,
            1,
            "team capability ceiling is empty or blocked by policy",
        )?;
        push_blocker(
            &mut blockers,
            !snapshot.conflicts.is_empty(),
            TeamMergeBlockerKind::Conflict,
            snapshot.conflicts.len() as u32,
            "merge readiness is blocked by unresolved worktree conflicts",
        )?;
        push_blocker(
            &mut blockers,
            snapshot.pending_approval_count > 0,
            TeamMergeBlockerKind::PendingApproval,
            snapshot.pending_approval_count,
            "merge readiness is blocked by pending approvals",
        )?;
        push_blocker(
            &mut blockers,
            stale_worktree_count > 0,
            TeamMergeBlockerKind::StaleWorktree,
            stale_worktree_count,
            "merge readiness is blocked by stale worktree leases",
        )?;
        push_blocker(
            &mut blockers,
            waiting_handoff_count > 0,
            TeamMergeBlockerKind::WaitingOnSubagent,
            waiting_handoff_count,
            "merge readiness is waiting on non-terminal handoffs",
        )?;
        push_blocker(
            &mut blockers,
            failed_handoff_count > 0,
            TeamMergeBlockerKind::FailedHandoff,
            failed_handoff_count,
            "merge readiness is blocked by failed or cancelled handoffs",
        )?;
        push_blocker(
            &mut blockers,
            snapshot.merge_review.blocking_review_count > 0,
            TeamMergeBlockerKind::BlockingReview,
            snapshot.merge_review.blocking_review_count,
            "merge readiness is blocked by reviewer votes",
        )?;

        let missing_review_count = snapshot
            .merge_review
            .required_review_count
            .saturating_sub(snapshot.merge_review.approving_review_count);
        push_blocker(
            &mut blockers,
            !snapshot.handoffs.is_empty() && missing_review_count > 0,
            TeamMergeBlockerKind::MissingReview,
            missing_review_count,
            "merge readiness requires additional approving reviews",
        )?;
        push_blocker(
            &mut blockers,
            !snapshot.handoffs.is_empty() && snapshot.merge_review.artifact_count == 0,
            TeamMergeBlockerKind::MissingArtifact,
            1,
            "merge readiness requires reviewed artifact evidence",
        )?;

        Ok(Self {
            team_id: snapshot.team_id,
            task_id: snapshot.task_id,
            readiness: snapshot.readiness,
            merge_ready: snapshot.readiness == TeamWorkflowReadiness::MergeReady
                && blockers.is_empty(),
            pending_approval_count: snapshot.pending_approval_count,
            conflict_count: snapshot.conflicts.len() as u32,
            stale_worktree_count,
            required_review_count: snapshot.merge_review.required_review_count,
            approving_review_count: snapshot.merge_review.approving_review_count,
            blocking_review_count: snapshot.merge_review.blocking_review_count,
            artifact_count: snapshot.merge_review.artifact_count,
            blockers,
            diagnostics: snapshot.diagnostics.clone(),
        })
    }
}

fn push_blocker(
    blockers: &mut Vec<TeamMergeBlocker>,
    condition: bool,
    kind: TeamMergeBlockerKind,
    count: u32,
    message: &'static str,
) -> Result<(), TeamError> {
    if condition {
        blockers.push(TeamMergeBlocker::new(kind, count, message)?);
    }
    Ok(())
}

fn merge_text(label: &str, value: String) -> Result<String, TeamError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TeamError::InvalidWorkflow(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(bounded_redacted(value, BLOCKER_MESSAGE_LIMIT))
}

fn bounded_redacted(value: &str, limit: usize) -> String {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() <= limit {
        return redacted;
    }
    let mut output = redacted.chars().take(limit).collect::<String>();
    output.push_str("...");
    output
}

fn redact_secret_like_text(input: &str) -> String {
    let mut output = String::new();
    let mut token = String::new();
    for ch in input.chars() {
        if ch.is_whitespace() || matches!(ch, '/' | '\\' | ':' | '=' | ',' | ';') {
            push_redacted_token(&mut output, &mut token);
            output.push(ch);
        } else {
            token.push(ch);
        }
    }
    push_redacted_token(&mut output, &mut token);
    output
}

fn push_redacted_token(output: &mut String, token: &mut String) {
    if token.is_empty() {
        return;
    }
    if is_secret_like(token) {
        output.push_str("[redacted]");
    } else {
        output.push_str(token);
    }
    token.clear();
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("secret")
        || value.contains("token")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.contains("password")
        || value.starts_with("sk-")
}
