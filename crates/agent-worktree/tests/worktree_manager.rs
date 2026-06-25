use agent_types::{TaskId, Timestamp, WorkspaceId};
use agent_worktree::{
    InMemoryWorktreeManager, WorktreeCleanupPolicy, WorktreeError, WorktreeLeaseId,
    WorktreeLeaseState, WorktreeManager, WorktreeRequest,
};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn request(path: &str) -> WorktreeRequest {
    WorktreeRequest::new(
        WorkspaceId::new(),
        TaskId::new(),
        "main",
        "agent/m09-01",
        path,
        WorktreeCleanupPolicy::DeleteOnRelease,
    )
    .expect("valid request")
}

#[test]
fn worktree_manager_serializes_lease_shape() {
    let mut manager = InMemoryWorktreeManager::default();
    let lease = manager
        .plan(request(".worktrees/agent-m09-01"), ts(0))
        .unwrap();

    let value = serde_json::to_value(lease).expect("serialize worktree lease");

    assert_eq!(value["id"], "worktree-1");
    assert_eq!(value["base_branch"], "main");
    assert_eq!(value["worktree_branch"], "agent/m09-01");
    assert_eq!(value["worktree_path"], ".worktrees/agent-m09-01");
    assert_eq!(value["state"], "planned");
    assert_eq!(value["created_at"], "2026-06-25T00:00:00Z");
}

#[test]
fn worktree_manager_plans_activates_and_releases_lease() {
    let mut manager = InMemoryWorktreeManager::default();
    let planned = manager
        .plan(request(".worktrees/agent-m09-01"), ts(0))
        .unwrap();
    let active = manager.activate(&planned.id, ts(1)).unwrap();
    let released = manager.release(&planned.id, ts(2)).unwrap();

    assert_eq!(planned.state, WorktreeLeaseState::Planned);
    assert_eq!(active.state, WorktreeLeaseState::Active);
    assert_eq!(released.state, WorktreeLeaseState::Released);
    assert_eq!(
        manager.get(&planned.id).expect("stored lease").state,
        WorktreeLeaseState::Released
    );
    assert_eq!(released.updated_at, ts(2));
}

#[test]
fn worktree_manager_rejects_invalid_request() {
    let empty_branch = WorktreeRequest::new(
        WorkspaceId::new(),
        TaskId::new(),
        " ",
        "agent/m09-01",
        ".worktrees/agent-m09-01",
        WorktreeCleanupPolicy::DeleteOnRelease,
    );
    let parent_traversal = WorktreeRequest::new(
        WorkspaceId::new(),
        TaskId::new(),
        "main",
        "agent/m09-01",
        "../outside",
        WorktreeCleanupPolicy::DeleteOnRelease,
    );

    assert!(matches!(
        empty_branch,
        Err(WorktreeError::InvalidRequest(_))
    ));
    assert!(matches!(
        parent_traversal,
        Err(WorktreeError::InvalidRequest(_))
    ));
}

#[test]
fn worktree_manager_rejects_unknown_lease() {
    let mut manager = InMemoryWorktreeManager::default();
    let missing = WorktreeLeaseId::new("worktree-missing").unwrap();

    let err = manager.activate(&missing, ts(1)).unwrap_err();

    assert!(matches!(err, WorktreeError::LeaseNotFound(_)));
}

#[test]
fn worktree_manager_rejects_duplicate_release() {
    let mut manager = InMemoryWorktreeManager::default();
    let lease = manager
        .plan(request(".worktrees/agent-m09-01"), ts(0))
        .unwrap();

    manager.release(&lease.id, ts(1)).unwrap();
    let err = manager.release(&lease.id, ts(2)).unwrap_err();

    assert!(matches!(err, WorktreeError::InvalidTransition(_)));
}
