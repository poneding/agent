use ruse_memory::{
    AuditedMemoryStore, InMemoryMemoryStore, MemoryAuditAction, MemoryRecord, MemoryScope,
    MemorySensitivity, MemorySource, MemoryStore,
};
use ruse_types::WorkspaceId;

fn record(id: &str, scope: MemoryScope, text: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope,
        text: text.to_string(),
        source: MemorySource {
            event_id: None,
            description: "project note".to_string(),
        },
        confidence: 80,
        sensitivity: MemorySensitivity::Internal,
    }
}

fn project_scope(workspace_id: WorkspaceId) -> MemoryScope {
    MemoryScope::Project { workspace_id }
}

#[test]
fn project_memory_adds_lists_and_deletes_by_workspace() {
    let scope = project_scope(WorkspaceId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record("project-memory", scope.clone(), "project fact"))
        .expect("add project memory");
    assert_eq!(store.list(&scope).expect("list project").len(), 1);

    assert!(
        store
            .delete(&scope, "project-memory")
            .expect("delete project memory")
    );
    assert!(store.list(&scope).expect("list project").is_empty());
}

#[test]
fn project_memory_isolates_workspaces() {
    let first_scope = project_scope(WorkspaceId::new());
    let second_scope = project_scope(WorkspaceId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record("memory", first_scope.clone(), "first project"))
        .expect("add first");
    store
        .add(record("memory", second_scope.clone(), "second project"))
        .expect("add second");

    assert_eq!(
        store.list(&first_scope).expect("list first")[0].text,
        "first project"
    );
    assert_eq!(
        store.list(&second_scope).expect("list second")[0].text,
        "second project"
    );
}

#[test]
fn project_memory_records_add_and_delete_audit_entries() {
    let scope = project_scope(WorkspaceId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record("project-memory", scope.clone(), "project fact"))
        .expect("add project memory");
    store
        .delete(&scope, "project-memory")
        .expect("delete project memory");

    let audit = store.audit_entries();

    assert_eq!(audit.len(), 2);
    assert_eq!(audit[0].action, MemoryAuditAction::Added);
    assert_eq!(audit[0].record_id, "project-memory");
    assert_eq!(audit[0].scope, scope);
    assert_eq!(audit[1].action, MemoryAuditAction::Deleted);
    assert_eq!(audit[1].record_id, "project-memory");
}

#[test]
fn project_memory_does_not_audit_failed_validation() {
    let scope = project_scope(WorkspaceId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record("project-memory", scope, " "))
        .expect_err("invalid memory rejected");

    assert!(store.audit_entries().is_empty());
}
