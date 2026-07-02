use agent_policy::{CapabilityAction, ResourceKind, ResourceScope};
use agent_team::{
    CapabilityGrant, CapabilitySet, HandoffStatus, ReviewVote, SubagentRole,
    TeamBlackboardEntrySummary, TeamBlackboardSummary, TeamCapabilityCeilingSummary,
    TeamHandoffSummary, TeamMergeReviewSummary, TeamWorkflowReadiness, TeamWorkflowSnapshot,
    TeamWorkflowStatus, TeamWorktreeState, TeamWorktreeSummary,
};
use agent_types::{
    AgentId, RecoveryConflictKind, RecoveryConflictNextAction, RecoveryConflictSide,
    RecoveryConflictSummary, TaskId, TeamId, WorkspaceId,
};

fn grant(
    action: CapabilityAction,
    kind: ResourceKind,
    pattern: &str,
) -> Result<CapabilityGrant, agent_team::TeamError> {
    CapabilityGrant::new(action, ResourceScope::new(kind, pattern).unwrap())
}

fn capability_set(grants: Vec<CapabilityGrant>) -> CapabilitySet {
    CapabilitySet::new(grants).expect("valid capability set")
}

fn team_id() -> TeamId {
    "00000000-0000-0000-0000-000000000063"
        .parse()
        .expect("valid team id")
}

fn lead_agent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000064"
        .parse()
        .expect("valid agent id")
}

fn subagent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000065"
        .parse()
        .expect("valid agent id")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000066"
        .parse()
        .expect("valid workspace id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000000067"
        .parse()
        .expect("valid task id")
}

fn worktree(state: TeamWorktreeState) -> TeamWorktreeSummary {
    TeamWorktreeSummary::new(
        "worktree-1",
        "main",
        "agent/m63-01",
        ".worktrees/agent-m63-01",
        state,
    )
    .expect("valid worktree")
}

fn completed_handoff() -> TeamHandoffSummary {
    TeamHandoffSummary::new(
        subagent_id(),
        SubagentRole::Implementer,
        "worktree-1",
        HandoffStatus::Completed,
        "Implement snapshot contracts",
    )
    .expect("valid handoff")
}

fn waiting_handoff() -> TeamHandoffSummary {
    TeamHandoffSummary::new(
        subagent_id(),
        SubagentRole::Implementer,
        "worktree-1",
        HandoffStatus::Accepted,
        "Implement snapshot contracts",
    )
    .expect("valid handoff")
}

fn blackboard(vote: Option<ReviewVote>) -> TeamBlackboardSummary {
    TeamBlackboardSummary::new(vec![
        TeamBlackboardEntrySummary::new(
            agent_team::BlackboardEntryKind::Review,
            "Reviewer approved snapshot contracts",
            1,
            2,
            vote,
        )
        .expect("valid entry"),
    ])
    .expect("valid blackboard")
}

fn approved_review() -> TeamMergeReviewSummary {
    TeamMergeReviewSummary::new(1, 1, 0, 2).expect("valid review")
}

fn missing_review() -> TeamMergeReviewSummary {
    TeamMergeReviewSummary::new(1, 0, 0, 2).expect("valid review")
}

fn ceiling() -> TeamCapabilityCeilingSummary {
    TeamCapabilityCeilingSummary::from_capability_set(
        &capability_set(vec![
            grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
        ]),
        false,
    )
}

fn snapshot(
    handoffs: Vec<TeamHandoffSummary>,
    review: TeamMergeReviewSummary,
    conflicts: Vec<RecoveryConflictSummary>,
) -> TeamWorkflowSnapshot {
    TeamWorkflowSnapshot::new(
        team_id(),
        lead_agent_id(),
        workspace_id(),
        task_id(),
        vec![worktree(TeamWorktreeState::Active)],
        handoffs,
        blackboard(Some(ReviewVote::Approve)),
        review,
        ceiling(),
        conflicts,
        Vec::new(),
    )
    .expect("valid snapshot")
}

#[test]
fn team_workflow_snapshot_serializes_stable_shape() {
    let snapshot = snapshot(vec![completed_handoff()], approved_review(), Vec::new());

    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(value["team_id"], "00000000-0000-0000-0000-000000000063");
    assert_eq!(value["status"], "merge_ready");
    assert_eq!(value["readiness"], "merge_ready");
    assert_eq!(value["worktrees"][0]["state"], "active");
    assert_eq!(value["handoffs"][0]["role"], "implementer");
    assert_eq!(value["handoffs"][0]["status"], "completed");
    assert_eq!(value["blackboard"]["entry_count"], 1);
    assert_eq!(value["merge_review"]["approved"], true);
    assert_eq!(value["capability_ceiling"]["grant_count"], 1);
}

#[test]
fn team_workflow_snapshot_reports_merge_ready_when_reviewed_without_conflicts() {
    let snapshot = snapshot(vec![completed_handoff()], approved_review(), Vec::new());

    assert_eq!(snapshot.status, TeamWorkflowStatus::MergeReady);
    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::MergeReady);
}

#[test]
fn team_workflow_snapshot_conflicts_take_precedence_over_merge_ready() {
    let snapshot = snapshot(
        vec![completed_handoff()],
        approved_review(),
        vec![RecoveryConflictSummary {
            path: "src/lib.rs".to_string(),
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
        }],
    );

    assert_eq!(snapshot.status, TeamWorkflowStatus::Blocked);
    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::HasConflict);
}

#[test]
fn team_workflow_snapshot_missing_review_blocks_merge_readiness() {
    let snapshot = snapshot(vec![completed_handoff()], missing_review(), Vec::new());

    assert_eq!(snapshot.status, TeamWorkflowStatus::ReviewRequired);
    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::ReviewRequired);
}

#[test]
fn team_workflow_snapshot_waits_on_non_terminal_handoff() {
    let snapshot = snapshot(vec![waiting_handoff()], approved_review(), Vec::new());

    assert_eq!(snapshot.status, TeamWorkflowStatus::InProgress);
    assert_eq!(snapshot.readiness, TeamWorkflowReadiness::WaitingOnSubagent);
}

#[test]
fn team_workflow_capability_ceiling_never_expands_parent_grants() {
    let parent = capability_set(vec![
        grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
    ]);
    let role = capability_set(vec![
        grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
        grant(
            CapabilityAction::Execute,
            ResourceKind::Command,
            "cargo test",
        )
        .unwrap(),
    ]);
    let task = role.clone();

    let ceiling = TeamCapabilityCeilingSummary::from_reduced_sets(&parent, &role, &task, false);

    assert_eq!(ceiling.grant_count, 1);
    assert_eq!(ceiling.grants[0].action, "read");
    assert_eq!(ceiling.grants[0].scope_kind, "file");
    assert_eq!(ceiling.grants[0].scope_pattern, "src/**");
}

#[test]
fn team_workflow_snapshot_redacts_secret_like_values() {
    let snapshot = TeamWorkflowSnapshot::new(
        team_id(),
        lead_agent_id(),
        workspace_id(),
        task_id(),
        vec![
            TeamWorktreeSummary::new(
                "worktree-secret-token",
                "main",
                "agent/api_key",
                ".worktrees/password-worktree",
                TeamWorktreeState::Active,
            )
            .expect("valid worktree"),
        ],
        vec![
            TeamHandoffSummary::new(
                subagent_id(),
                SubagentRole::Reviewer,
                "worktree-secret-token",
                HandoffStatus::Completed,
                "review password token",
            )
            .expect("valid handoff"),
        ],
        TeamBlackboardSummary::new(vec![
            TeamBlackboardEntrySummary::new(
                agent_team::BlackboardEntryKind::Review,
                "api_key leaked in review",
                1,
                1,
                Some(ReviewVote::Approve),
            )
            .expect("valid entry"),
        ])
        .expect("valid blackboard"),
        approved_review(),
        TeamCapabilityCeilingSummary::from_capability_set(
            &capability_set(vec![
                grant(
                    CapabilityAction::Read,
                    ResourceKind::File,
                    "src/token-secret.rs",
                )
                .unwrap(),
            ]),
            false,
        ),
        Vec::new(),
        Vec::new(),
    )
    .expect("valid snapshot");

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(serialized.contains("[redacted]"));
    for forbidden in ["secret", "api_key", "password", "token"] {
        assert!(
            !serialized.contains(forbidden),
            "serialized team workflow snapshot leaked {forbidden}: {serialized}"
        );
    }
}
