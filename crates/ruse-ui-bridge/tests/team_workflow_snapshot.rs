use ruse_policy::{CapabilityAction, ResourceKind, ResourceScope};
use ruse_team::{
    CapabilityGrant, CapabilitySet, HandoffStatus, ReviewVote, SubagentRole,
    TeamBlackboardEntrySummary, TeamBlackboardSummary, TeamCapabilityCeilingSummary,
    TeamHandoffSummary, TeamMergeReviewSummary, TeamWorkflowReadiness, TeamWorkflowSnapshot,
    TeamWorktreeState, TeamWorktreeSummary,
};
use ruse_types::{AgentId, TaskId, TeamId, WorkspaceId};
use ruse_ui_bridge::{UiJsonFrame, UiTeamWorkflowSnapshot, encode_json_frame};

fn team_id() -> TeamId {
    "00000000-0000-0000-0000-000000006330"
        .parse()
        .expect("valid team id")
}

fn agent_id(value: &str) -> AgentId {
    value.parse().expect("valid agent id")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000006331"
        .parse()
        .expect("valid workspace id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000006332"
        .parse()
        .expect("valid task id")
}

fn snapshot() -> TeamWorkflowSnapshot {
    let capabilities = CapabilitySet::new(vec![
        CapabilityGrant::new(
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::File, "crates/ruse-team/**").unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();

    TeamWorkflowSnapshot::new(
        team_id(),
        agent_id("00000000-0000-0000-0000-000000006333"),
        workspace_id(),
        task_id(),
        vec![
            TeamWorktreeSummary::new(
                "worktree-ui",
                "main",
                "agent/m63-02",
                ".worktrees/agent-m63-02",
                TeamWorktreeState::Active,
            )
            .unwrap(),
        ],
        vec![
            TeamHandoffSummary::new(
                agent_id("00000000-0000-0000-0000-000000006334"),
                SubagentRole::Implementer,
                "worktree-ui",
                HandoffStatus::Completed,
                "Implement team UI bridge snapshot",
            )
            .unwrap(),
        ],
        TeamBlackboardSummary::new(vec![
            TeamBlackboardEntrySummary::new(
                ruse_team::BlackboardEntryKind::Review,
                "Reviewer approved bridge snapshot",
                1,
                1,
                Some(ReviewVote::Approve),
            )
            .unwrap(),
        ])
        .unwrap(),
        TeamMergeReviewSummary::new(1, 1, 0, 1).unwrap(),
        TeamCapabilityCeilingSummary::from_capability_set(&capabilities, false),
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
}

#[test]
fn team_workflow_snapshot_frame_has_stable_json_tag() {
    let frame = UiJsonFrame::team_workflow_snapshot(
        UiTeamWorkflowSnapshot::new("merge_readiness_dry_run", true, snapshot(), Vec::new())
            .expect("valid ui frame"),
    );

    let encoded = encode_json_frame(&frame).expect("encode frame");
    let value: serde_json::Value = serde_json::from_str(&encoded).expect("json frame");

    assert_eq!(value["type"], "team_workflow_snapshot");
    assert_eq!(value["data"]["operation"], "merge_readiness_dry_run");
    assert_eq!(value["data"]["dry_run"], true);
    assert_eq!(value["data"]["snapshot"]["readiness"], "merge_ready");
    assert_eq!(
        value["data"]["snapshot"]["capability_ceiling"]["grant_count"],
        1
    );

    let decoded: UiJsonFrame = serde_json::from_str(&encoded).expect("decode frame");
    assert!(matches!(
        decoded,
        UiJsonFrame::team_workflow_snapshot(frame)
            if frame.snapshot.readiness == TeamWorkflowReadiness::MergeReady
                && frame.errors.is_empty()
    ));
}
