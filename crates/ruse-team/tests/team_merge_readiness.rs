use ruse_policy::{CapabilityAction, ResourceKind, ResourceScope};
use ruse_team::{
    CapabilityGrant, CapabilitySet, HandoffStatus, ReviewVote, SubagentRole,
    TeamBlackboardEntrySummary, TeamBlackboardSummary, TeamCapabilityCeilingSummary,
    TeamHandoffSummary, TeamMergeBlocker, TeamMergeBlockerKind, TeamMergeReadinessReport,
    TeamMergeReviewSummary, TeamWorkflowDiagnostic, TeamWorkflowDiagnosticSeverity,
    TeamWorkflowReadiness, TeamWorkflowSnapshot, TeamWorktreeState, TeamWorktreeSummary,
};
use ruse_types::{
    AgentId, RecoveryConflictKind, RecoveryConflictNextAction, RecoveryConflictSide,
    RecoveryConflictSummary, TaskId, TeamId, WorkspaceId,
};

fn team_id() -> TeamId {
    "00000000-0000-0000-0000-000000000163"
        .parse()
        .expect("valid team id")
}

fn lead_agent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000164"
        .parse()
        .expect("valid agent id")
}

fn subagent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000165"
        .parse()
        .expect("valid agent id")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000166"
        .parse()
        .expect("valid workspace id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000000167"
        .parse()
        .expect("valid task id")
}

fn capability_set() -> CapabilitySet {
    CapabilitySet::new(vec![
        CapabilityGrant::new(
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::File, "crates/ruse-team/**").unwrap(),
        )
        .unwrap(),
    ])
    .unwrap()
}

fn ceiling(policy_blocked: bool) -> TeamCapabilityCeilingSummary {
    if policy_blocked {
        TeamCapabilityCeilingSummary {
            grant_count: 0,
            grants: Vec::new(),
            policy_blocked: true,
        }
    } else {
        TeamCapabilityCeilingSummary::from_capability_set(&capability_set(), false)
    }
}

fn worktree(state: TeamWorktreeState) -> TeamWorktreeSummary {
    TeamWorktreeSummary::new(
        "worktree-merge",
        "main",
        "agent/m63-03",
        ".worktrees/agent-m63-03",
        state,
    )
    .unwrap()
}

fn handoff(status: HandoffStatus) -> TeamHandoffSummary {
    TeamHandoffSummary::new(
        subagent_id(),
        SubagentRole::Implementer,
        "worktree-merge",
        status,
        "Implement merge readiness",
    )
    .unwrap()
}

fn blackboard(vote: Option<ReviewVote>) -> TeamBlackboardSummary {
    TeamBlackboardSummary::new(vec![
        TeamBlackboardEntrySummary::new(
            ruse_team::BlackboardEntryKind::Review,
            "Reviewer checked merge readiness",
            1,
            1,
            vote,
        )
        .unwrap(),
    ])
    .unwrap()
}

fn conflict() -> RecoveryConflictSummary {
    RecoveryConflictSummary {
        path: "crates/ruse-team/src/workflow.rs".to_string(),
        kind: RecoveryConflictKind::BothModified,
        lead_change: RecoveryConflictSide {
            source: "lead".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("lead-hash".to_string()),
        },
        incoming_change: RecoveryConflictSide {
            source: "subagent".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("incoming-hash".to_string()),
        },
        next_action: RecoveryConflictNextAction::InspectDiff,
        diagnostics: Vec::new(),
    }
}

fn snapshot(
    worktree_state: TeamWorktreeState,
    handoff_status: HandoffStatus,
    review: TeamMergeReviewSummary,
    capability_ceiling: TeamCapabilityCeilingSummary,
    pending_approval_count: u32,
    conflicts: Vec<RecoveryConflictSummary>,
) -> TeamWorkflowSnapshot {
    TeamWorkflowSnapshot::new_with_pending_approvals(
        team_id(),
        lead_agent_id(),
        workspace_id(),
        task_id(),
        vec![worktree(worktree_state)],
        vec![handoff(handoff_status)],
        blackboard(Some(ReviewVote::Approve)),
        review,
        capability_ceiling,
        pending_approval_count,
        conflicts,
        vec![
            TeamWorkflowDiagnostic::new(
                TeamWorkflowDiagnosticSeverity::Info,
                "team.merge.fixture",
                "merge readiness fixture",
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn approved_review(artifact_count: u32) -> TeamMergeReviewSummary {
    TeamMergeReviewSummary::new(1, 1, 0, artifact_count).unwrap()
}

fn missing_review() -> TeamMergeReviewSummary {
    TeamMergeReviewSummary::new(1, 0, 0, 1).unwrap()
}

fn blocking_review() -> TeamMergeReviewSummary {
    TeamMergeReviewSummary::new(1, 0, 1, 1).unwrap()
}

#[test]
fn merge_readiness_report_is_ready_without_blockers() {
    let snapshot = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        approved_review(1),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&snapshot).unwrap();

    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::MergeReady);
    assert!(report.merge_ready);
    assert!(report.blockers.is_empty());
    assert_eq!(report.artifact_count, 1);
}

#[test]
fn merge_readiness_report_blocks_on_conflicts() {
    let snapshot = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        approved_review(1),
        ceiling(false),
        0,
        vec![conflict()],
    );
    let report = TeamMergeReadinessReport::from_snapshot(&snapshot).unwrap();

    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::HasConflict);
    assert!(!report.merge_ready);
    assert_eq!(report.conflict_count, 1);
    assert_eq!(report.blockers[0].kind, TeamMergeBlockerKind::Conflict);
}

#[test]
fn merge_readiness_report_blocks_on_pending_approval() {
    let snapshot = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        approved_review(1),
        ceiling(false),
        2,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&snapshot).unwrap();

    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::Blocked);
    assert!(!report.merge_ready);
    assert_eq!(report.pending_approval_count, 2);
    assert_eq!(
        report.blockers[0].kind,
        TeamMergeBlockerKind::PendingApproval
    );
}

#[test]
fn merge_readiness_report_blocks_on_stale_worktree() {
    let snapshot = snapshot(
        TeamWorktreeState::Stale,
        HandoffStatus::Completed,
        approved_review(1),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&snapshot).unwrap();

    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::Blocked);
    assert_eq!(report.stale_worktree_count, 1);
    assert_eq!(report.blockers[0].kind, TeamMergeBlockerKind::StaleWorktree);
}

#[test]
fn merge_readiness_report_blocks_on_review_states() {
    let blocking = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        blocking_review(),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&blocking).unwrap();
    assert_eq!(blocking.readiness, TeamWorkflowReadiness::Blocked);
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.kind == TeamMergeBlockerKind::BlockingReview)
    );

    let missing = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        missing_review(),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&missing).unwrap();
    assert_eq!(missing.readiness, TeamWorkflowReadiness::ReviewRequired);
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.kind == TeamMergeBlockerKind::MissingReview)
    );
}

#[test]
fn merge_readiness_report_blocks_on_missing_artifacts_and_policy() {
    let missing_artifact = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        approved_review(0),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&missing_artifact).unwrap();
    assert_eq!(
        missing_artifact.readiness,
        TeamWorkflowReadiness::ReviewRequired
    );
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.kind == TeamMergeBlockerKind::MissingArtifact)
    );

    let policy = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Completed,
        approved_review(1),
        ceiling(true),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&policy).unwrap();
    assert_eq!(policy.readiness, TeamWorkflowReadiness::PolicyBlocked);
    assert_eq!(report.blockers[0].kind, TeamMergeBlockerKind::PolicyBlocked);
}

#[test]
fn merge_readiness_report_blocks_on_waiting_or_failed_handoff() {
    let waiting = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Accepted,
        approved_review(1),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&waiting).unwrap();
    assert_eq!(waiting.readiness, TeamWorkflowReadiness::WaitingOnSubagent);
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.kind == TeamMergeBlockerKind::WaitingOnSubagent)
    );

    let failed = snapshot(
        TeamWorktreeState::Active,
        HandoffStatus::Failed,
        approved_review(1),
        ceiling(false),
        0,
        Vec::new(),
    );
    let report = TeamMergeReadinessReport::from_snapshot(&failed).unwrap();
    assert_eq!(failed.readiness, TeamWorkflowReadiness::Blocked);
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.kind == TeamMergeBlockerKind::FailedHandoff)
    );
}

#[test]
fn merge_blocker_redacts_secret_like_messages() {
    let blocker = TeamMergeBlocker::new(
        TeamMergeBlockerKind::PolicyBlocked,
        1,
        "api_key sk-secret-token password",
    )
    .unwrap();

    assert!(blocker.message.contains("[redacted]"));
    for forbidden in ["api_key", "secret", "token", "password"] {
        assert!(
            !blocker.message.contains(forbidden),
            "merge blocker leaked {forbidden}: {}",
            blocker.message
        );
    }
}
