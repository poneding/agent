use ruse_policy::{CapabilityAction, ResourceKind, ResourceScope};
use ruse_team::{
    CapabilityGrant, CapabilitySet, HandoffEnvelope, HandoffStatus, SubagentProfile, SubagentRole,
    TeamError, reduce_capabilities, transition_handoff_status,
};
use ruse_types::{AgentId, TaskId, Timestamp};

fn ts() -> Timestamp {
    serde_json::from_str("\"2026-06-25T00:00:00Z\"").expect("valid fixed timestamp")
}

fn nil_agent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000000"
        .parse()
        .expect("valid agent id")
}

fn nil_task_id() -> TaskId {
    "00000000-0000-0000-0000-000000000000"
        .parse()
        .expect("valid task id")
}

fn grant(
    action: CapabilityAction,
    kind: ResourceKind,
    pattern: &str,
) -> Result<CapabilityGrant, TeamError> {
    CapabilityGrant::new(action, ResourceScope::new(kind, pattern).unwrap())
}

fn capability_set(grants: Vec<CapabilityGrant>) -> Result<CapabilitySet, TeamError> {
    CapabilitySet::new(grants)
}

fn profile() -> Result<SubagentProfile, TeamError> {
    SubagentProfile::new(
        nil_agent_id(),
        SubagentRole::Implementer,
        "Implementation agent",
        "Focused Rust implementation worker",
    )
}

#[test]
fn subagent_handoff_serializes_envelope_shape() {
    let capabilities = capability_set(vec![
        grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
    ])
    .unwrap();
    let envelope = HandoffEnvelope::new(
        nil_task_id(),
        nil_agent_id(),
        profile().unwrap(),
        "worktree-1",
        capabilities,
        "Implement the handoff contracts",
        ts(),
    )
    .unwrap();

    let value = serde_json::to_value(envelope).expect("serialize handoff envelope");

    assert_eq!(value["task_id"], "00000000-0000-0000-0000-000000000000");
    assert_eq!(
        value["parent_agent_id"],
        "00000000-0000-0000-0000-000000000000"
    );
    assert_eq!(value["subagent_profile"]["role"], "implementer");
    assert_eq!(
        value["subagent_profile"]["display_name"],
        "Implementation agent"
    );
    assert_eq!(value["worktree_lease_id"], "worktree-1");
    assert_eq!(value["capability_set"]["grants"][0]["action"], "read");
    assert_eq!(
        value["capability_set"]["grants"][0]["scope"]["kind"],
        "file"
    );
    assert_eq!(value["status"], "created");
    assert_eq!(value["objective"], "Implement the handoff contracts");
    assert_eq!(value["created_at"], "2026-06-25T00:00:00Z");
}

#[test]
fn subagent_handoff_reduces_capabilities_by_intersection() {
    let shared = grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap();
    let parent_only = grant(
        CapabilityAction::Execute,
        ResourceKind::Command,
        "cargo test",
    )
    .unwrap();
    let role_only = grant(CapabilityAction::Search, ResourceKind::Workspace, "**").unwrap();
    let task_only = grant(CapabilityAction::Read, ResourceKind::File, "docs/**").unwrap();

    let parent = capability_set(vec![shared.clone(), parent_only]).unwrap();
    let role = capability_set(vec![shared.clone(), role_only]).unwrap();
    let task = capability_set(vec![shared.clone(), task_only]).unwrap();

    let reduced = reduce_capabilities(&parent, &role, &task);

    assert_eq!(reduced.grants, vec![shared]);
}

#[test]
fn subagent_handoff_never_expands_parent_capabilities() {
    let parent = capability_set(vec![
        grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
    ])
    .unwrap();
    let role = capability_set(vec![
        grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
        grant(
            CapabilityAction::Execute,
            ResourceKind::Command,
            "cargo test",
        )
        .unwrap(),
    ])
    .unwrap();
    let task = role.clone();

    let reduced = reduce_capabilities(&parent, &role, &task);

    assert_eq!(reduced.grants.len(), 1);
    assert!(reduced.grants.iter().all(|grant| parent.contains(grant)));
}

#[test]
fn subagent_handoff_rejects_invalid_inputs() {
    let empty_name = SubagentProfile::new(
        AgentId::new(),
        SubagentRole::Reviewer,
        " ",
        "Reviews implementation quality",
    );
    let empty_summary =
        SubagentProfile::new(AgentId::new(), SubagentRole::Researcher, "Researcher", " ");
    let empty_capability_set = CapabilitySet::new(Vec::new());

    assert!(matches!(empty_name, Err(TeamError::InvalidHandoff(_))));
    assert!(matches!(empty_summary, Err(TeamError::InvalidHandoff(_))));
    assert!(matches!(
        empty_capability_set,
        Err(TeamError::InvalidHandoff(_))
    ));

    let err = HandoffEnvelope::new(
        TaskId::new(),
        AgentId::new(),
        profile().unwrap(),
        " ",
        capability_set(vec![
            grant(CapabilityAction::Read, ResourceKind::File, "src/**").unwrap(),
        ])
        .unwrap(),
        " ",
        ts(),
    )
    .unwrap_err();

    assert!(matches!(err, TeamError::InvalidHandoff(_)));
}

#[test]
fn subagent_handoff_rejects_terminal_status_transition() {
    let completed = transition_handoff_status(
        HandoffStatus::Accepted,
        HandoffStatus::Completed,
        "subagent finished",
    )
    .unwrap();

    let err = transition_handoff_status(completed, HandoffStatus::Failed, "late failure")
        .expect_err("terminal handoff cannot transition");

    assert!(matches!(err, TeamError::InvalidTransition(_)));
}
