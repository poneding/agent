use agent_policy::{CapabilityAction, ResourceKind};
use agent_types::{
    AgentId, RecoveryConflictSide, RecoveryConflictSummary, RecoveryDiagnostic, TaskId, TeamId,
    WorkspaceId,
};
use serde::{Deserialize, Serialize};

use crate::{
    BlackboardEntryKind, CapabilitySet, HandoffStatus, ReviewVote, SubagentRole, TeamError,
    reduce_capabilities,
};

const SHORT_TEXT_LIMIT: usize = 120;
const SUMMARY_LIMIT: usize = 240;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamWorkflowStatus {
    Planning,
    HandoffReady,
    InProgress,
    ReviewRequired,
    MergeReady,
    Blocked,
    Completed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamWorkflowReadiness {
    ReadyForHandoff,
    WaitingOnSubagent,
    ReviewRequired,
    MergeReady,
    HasConflict,
    PolicyBlocked,
    Blocked,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamWorktreeState {
    Planned,
    Active,
    Released,
    CleanupFailed,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamWorktreeSummary {
    pub lease_id: String,
    pub base_branch: String,
    pub worktree_branch: String,
    pub worktree_path: String,
    pub state: TeamWorktreeState,
}

impl TeamWorktreeSummary {
    pub fn new(
        lease_id: impl Into<String>,
        base_branch: impl Into<String>,
        worktree_branch: impl Into<String>,
        worktree_path: impl Into<String>,
        state: TeamWorktreeState,
    ) -> Result<Self, TeamError> {
        Ok(Self {
            lease_id: workflow_text("worktree lease id", lease_id.into(), SHORT_TEXT_LIMIT)?,
            base_branch: workflow_text("base branch", base_branch.into(), SHORT_TEXT_LIMIT)?,
            worktree_branch: workflow_text(
                "worktree branch",
                worktree_branch.into(),
                SHORT_TEXT_LIMIT,
            )?,
            worktree_path: workflow_text("worktree path", worktree_path.into(), SHORT_TEXT_LIMIT)?,
            state,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamHandoffSummary {
    pub subagent_id: AgentId,
    pub role: SubagentRole,
    pub worktree_lease_id: String,
    pub status: HandoffStatus,
    pub objective: String,
}

impl TeamHandoffSummary {
    pub fn new(
        subagent_id: AgentId,
        role: SubagentRole,
        worktree_lease_id: impl Into<String>,
        status: HandoffStatus,
        objective: impl Into<String>,
    ) -> Result<Self, TeamError> {
        Ok(Self {
            subagent_id,
            role,
            worktree_lease_id: workflow_text(
                "worktree lease id",
                worktree_lease_id.into(),
                SHORT_TEXT_LIMIT,
            )?,
            status,
            objective: workflow_text("handoff objective", objective.into(), SUMMARY_LIMIT)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamBlackboardEntrySummary {
    pub kind: BlackboardEntryKind,
    pub summary: String,
    pub artifact_count: u32,
    pub trace_event_count: u32,
    pub review_vote: Option<ReviewVote>,
}

impl TeamBlackboardEntrySummary {
    pub fn new(
        kind: BlackboardEntryKind,
        summary: impl Into<String>,
        artifact_count: u32,
        trace_event_count: u32,
        review_vote: Option<ReviewVote>,
    ) -> Result<Self, TeamError> {
        Ok(Self {
            kind,
            summary: workflow_text("blackboard summary", summary.into(), SUMMARY_LIMIT)?,
            artifact_count,
            trace_event_count,
            review_vote,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamBlackboardSummary {
    pub entry_count: u32,
    pub latest_entries: Vec<TeamBlackboardEntrySummary>,
    pub approve_count: u32,
    pub request_changes_count: u32,
    pub block_count: u32,
}

impl TeamBlackboardSummary {
    pub fn new(entries: Vec<TeamBlackboardEntrySummary>) -> Result<Self, TeamError> {
        let entry_count = entries.len() as u32;
        let approve_count = entries
            .iter()
            .filter(|entry| entry.review_vote == Some(ReviewVote::Approve))
            .count() as u32;
        let request_changes_count = entries
            .iter()
            .filter(|entry| entry.review_vote == Some(ReviewVote::RequestChanges))
            .count() as u32;
        let block_count = entries
            .iter()
            .filter(|entry| entry.review_vote == Some(ReviewVote::Block))
            .count() as u32;

        Ok(Self {
            entry_count,
            latest_entries: entries,
            approve_count,
            request_changes_count,
            block_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamMergeReviewSummary {
    pub required_review_count: u32,
    pub approving_review_count: u32,
    pub blocking_review_count: u32,
    pub artifact_count: u32,
    pub approved: bool,
}

impl TeamMergeReviewSummary {
    pub fn new(
        required_review_count: u32,
        approving_review_count: u32,
        blocking_review_count: u32,
        artifact_count: u32,
    ) -> Result<Self, TeamError> {
        Ok(Self {
            required_review_count,
            approving_review_count,
            blocking_review_count,
            artifact_count,
            approved: blocking_review_count == 0
                && approving_review_count >= required_review_count
                && artifact_count > 0,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamCapabilityGrantSummary {
    pub action: String,
    pub scope_kind: String,
    pub scope_pattern: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamCapabilityCeilingSummary {
    pub grant_count: u32,
    pub grants: Vec<TeamCapabilityGrantSummary>,
    pub policy_blocked: bool,
}

impl TeamCapabilityCeilingSummary {
    #[must_use]
    pub fn from_reduced_sets(
        parent: &CapabilitySet,
        role: &CapabilitySet,
        task: &CapabilitySet,
        policy_blocked: bool,
    ) -> Self {
        let reduced = reduce_capabilities(parent, role, task);
        Self::from_capability_set(&reduced, policy_blocked)
    }

    #[must_use]
    pub fn from_capability_set(capabilities: &CapabilitySet, policy_blocked: bool) -> Self {
        let grants = capabilities
            .grants
            .iter()
            .map(|grant| TeamCapabilityGrantSummary {
                action: capability_action_label(grant.action).to_string(),
                scope_kind: resource_kind_label(grant.scope.kind).to_string(),
                scope_pattern: bounded_redacted(&grant.scope.pattern, SHORT_TEXT_LIMIT),
            })
            .collect::<Vec<_>>();

        Self {
            grant_count: grants.len() as u32,
            grants,
            policy_blocked,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamWorkflowDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamWorkflowDiagnostic {
    pub severity: TeamWorkflowDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl TeamWorkflowDiagnostic {
    pub fn new(
        severity: TeamWorkflowDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, TeamError> {
        Ok(Self {
            severity,
            code: workflow_text("diagnostic code", code.into(), SHORT_TEXT_LIMIT)?,
            message: workflow_text("diagnostic message", message.into(), SUMMARY_LIMIT)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamWorkflowSnapshot {
    pub team_id: TeamId,
    pub lead_agent_id: AgentId,
    pub workspace_id: WorkspaceId,
    pub task_id: TaskId,
    pub status: TeamWorkflowStatus,
    pub readiness: TeamWorkflowReadiness,
    pub worktrees: Vec<TeamWorktreeSummary>,
    pub handoffs: Vec<TeamHandoffSummary>,
    pub blackboard: TeamBlackboardSummary,
    pub merge_review: TeamMergeReviewSummary,
    pub capability_ceiling: TeamCapabilityCeilingSummary,
    #[serde(default)]
    pub pending_approval_count: u32,
    pub conflicts: Vec<RecoveryConflictSummary>,
    pub diagnostics: Vec<TeamWorkflowDiagnostic>,
}

impl TeamWorkflowSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        team_id: TeamId,
        lead_agent_id: AgentId,
        workspace_id: WorkspaceId,
        task_id: TaskId,
        worktrees: Vec<TeamWorktreeSummary>,
        handoffs: Vec<TeamHandoffSummary>,
        blackboard: TeamBlackboardSummary,
        merge_review: TeamMergeReviewSummary,
        capability_ceiling: TeamCapabilityCeilingSummary,
        conflicts: Vec<RecoveryConflictSummary>,
        diagnostics: Vec<TeamWorkflowDiagnostic>,
    ) -> Result<Self, TeamError> {
        Self::new_with_pending_approvals(
            team_id,
            lead_agent_id,
            workspace_id,
            task_id,
            worktrees,
            handoffs,
            blackboard,
            merge_review,
            capability_ceiling,
            0,
            conflicts,
            diagnostics,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_pending_approvals(
        team_id: TeamId,
        lead_agent_id: AgentId,
        workspace_id: WorkspaceId,
        task_id: TaskId,
        worktrees: Vec<TeamWorktreeSummary>,
        handoffs: Vec<TeamHandoffSummary>,
        blackboard: TeamBlackboardSummary,
        merge_review: TeamMergeReviewSummary,
        capability_ceiling: TeamCapabilityCeilingSummary,
        pending_approval_count: u32,
        conflicts: Vec<RecoveryConflictSummary>,
        diagnostics: Vec<TeamWorkflowDiagnostic>,
    ) -> Result<Self, TeamError> {
        let conflicts = conflicts
            .into_iter()
            .map(redact_conflict)
            .collect::<Vec<_>>();
        let readiness = derive_readiness(
            &worktrees,
            &handoffs,
            &merge_review,
            &capability_ceiling,
            pending_approval_count,
            &conflicts,
        );
        let status = status_for_readiness(readiness);

        Ok(Self {
            team_id,
            lead_agent_id,
            workspace_id,
            task_id,
            status,
            readiness,
            worktrees,
            handoffs,
            blackboard,
            merge_review,
            capability_ceiling,
            pending_approval_count,
            conflicts,
            diagnostics,
        })
    }
}

fn derive_readiness(
    worktrees: &[TeamWorktreeSummary],
    handoffs: &[TeamHandoffSummary],
    merge_review: &TeamMergeReviewSummary,
    capability_ceiling: &TeamCapabilityCeilingSummary,
    pending_approval_count: u32,
    conflicts: &[RecoveryConflictSummary],
) -> TeamWorkflowReadiness {
    if capability_ceiling.policy_blocked || capability_ceiling.grant_count == 0 {
        return TeamWorkflowReadiness::PolicyBlocked;
    }
    if !conflicts.is_empty() {
        return TeamWorkflowReadiness::HasConflict;
    }
    if pending_approval_count > 0
        || worktrees
            .iter()
            .any(|worktree| worktree.state == TeamWorktreeState::Stale)
    {
        return TeamWorkflowReadiness::Blocked;
    }
    if handoffs.iter().any(|handoff| {
        matches!(
            handoff.status,
            HandoffStatus::Created | HandoffStatus::Accepted
        )
    }) {
        return TeamWorkflowReadiness::WaitingOnSubagent;
    }
    if handoffs.iter().any(|handoff| {
        matches!(
            handoff.status,
            HandoffStatus::Failed | HandoffStatus::Cancelled
        )
    }) || merge_review.blocking_review_count > 0
    {
        return TeamWorkflowReadiness::Blocked;
    }
    if !handoffs.is_empty() && !merge_review.approved {
        return TeamWorkflowReadiness::ReviewRequired;
    }
    if !handoffs.is_empty() && merge_review.approved {
        return TeamWorkflowReadiness::MergeReady;
    }
    TeamWorkflowReadiness::ReadyForHandoff
}

fn status_for_readiness(readiness: TeamWorkflowReadiness) -> TeamWorkflowStatus {
    match readiness {
        TeamWorkflowReadiness::ReadyForHandoff => TeamWorkflowStatus::HandoffReady,
        TeamWorkflowReadiness::WaitingOnSubagent => TeamWorkflowStatus::InProgress,
        TeamWorkflowReadiness::ReviewRequired => TeamWorkflowStatus::ReviewRequired,
        TeamWorkflowReadiness::MergeReady => TeamWorkflowStatus::MergeReady,
        TeamWorkflowReadiness::HasConflict
        | TeamWorkflowReadiness::PolicyBlocked
        | TeamWorkflowReadiness::Blocked => TeamWorkflowStatus::Blocked,
    }
}

fn redact_conflict(mut conflict: RecoveryConflictSummary) -> RecoveryConflictSummary {
    conflict.path = bounded_redacted(&conflict.path, SHORT_TEXT_LIMIT);
    conflict.lead_change = redact_conflict_side(conflict.lead_change);
    conflict.incoming_change = redact_conflict_side(conflict.incoming_change);
    conflict.diagnostics = conflict
        .diagnostics
        .into_iter()
        .map(redact_diagnostic)
        .collect();
    conflict
}

fn redact_conflict_side(mut side: RecoveryConflictSide) -> RecoveryConflictSide {
    side.source = bounded_redacted(&side.source, SHORT_TEXT_LIMIT);
    side.change_kind = bounded_redacted(&side.change_kind, SHORT_TEXT_LIMIT);
    side.content_hash = side
        .content_hash
        .map(|value| bounded_redacted(&value, SHORT_TEXT_LIMIT));
    side
}

fn redact_diagnostic(mut diagnostic: RecoveryDiagnostic) -> RecoveryDiagnostic {
    diagnostic.code = bounded_redacted(&diagnostic.code, SHORT_TEXT_LIMIT);
    diagnostic.message = bounded_redacted(&diagnostic.message, SUMMARY_LIMIT);
    diagnostic.source = diagnostic
        .source
        .map(|value| bounded_redacted(&value, SHORT_TEXT_LIMIT));
    diagnostic
}

fn workflow_text(label: &str, value: String, limit: usize) -> Result<String, TeamError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TeamError::InvalidWorkflow(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(bounded_redacted(value, limit))
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
        if is_token_separator(ch) {
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

fn is_token_separator(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '/' | '\\' | ':' | '=' | ',' | ';')
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

const fn capability_action_label(action: CapabilityAction) -> &'static str {
    match action {
        CapabilityAction::Read => "read",
        CapabilityAction::Search => "search",
        CapabilityAction::Execute => "execute",
        CapabilityAction::Network => "network",
    }
}

const fn resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::File => "file",
        ResourceKind::Workspace => "workspace",
        ResourceKind::Command => "command",
        ResourceKind::Network => "network",
        ResourceKind::Tool => "tool",
    }
}
