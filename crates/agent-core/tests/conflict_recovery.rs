use agent_core::{ConflictRecoveryMapper, RecoveryEngine};
use agent_storage::InMemoryEventStore;
use agent_types::{
    ConversationId, RecoveryConflictKind, RecoveryConflictNextAction, RecoveryReadiness, TurnId,
};
use agent_worktree::{ChangeKind, FileChange, detect_conflicts};

fn change(path: &str, kind: ChangeKind, hash: Option<&str>) -> FileChange {
    FileChange::new(path, kind, hash.map(str::to_string)).expect("valid file change")
}

#[test]
fn conflict_recovery_maps_report_to_sorted_safe_summaries() {
    let lead = vec![
        change(
            "src/token-secret.rs",
            ChangeKind::Modified,
            Some("lead-secret-hash"),
        ),
        change("src/a.rs", ChangeKind::Deleted, None),
    ];
    let incoming = vec![
        change("src/a.rs", ChangeKind::Modified, Some("incoming")),
        change(
            "src/token-secret.rs",
            ChangeKind::Modified,
            Some("incoming-api_key-hash"),
        ),
    ];
    let report = detect_conflicts(&lead, &incoming).expect("conflicts");

    let summaries = ConflictRecoveryMapper.map_report(&report);
    let serialized = serde_json::to_string(&summaries).expect("serialize summaries");

    assert_eq!(summaries.len(), 2);
    let delete_modify = summaries
        .iter()
        .find(|summary| summary.path == "src/a.rs")
        .expect("delete/modify conflict");
    assert_eq!(delete_modify.kind, RecoveryConflictKind::DeleteModify);
    assert_eq!(
        delete_modify.next_action,
        RecoveryConflictNextAction::ChooseSide
    );
    let both_modified = summaries
        .iter()
        .find(|summary| summary.path == "src/[redacted]")
        .expect("redacted both-modified conflict");
    assert_eq!(both_modified.kind, RecoveryConflictKind::BothModified);
    assert_eq!(both_modified.lead_change.change_kind, "modified");
    assert!(!serialized.contains("token-secret"));
    assert!(!serialized.contains("lead-secret-hash"));
    assert!(!serialized.contains("api_key"));
}

#[test]
fn conflict_recovery_blocks_snapshot_readiness() {
    let store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let report = detect_conflicts(
        &[change("src/lib.rs", ChangeKind::Modified, Some("lead"))],
        &[change("src/lib.rs", ChangeKind::Modified, Some("incoming"))],
    )
    .expect("conflict report");
    let conflicts = ConflictRecoveryMapper.map_report(&report);

    let snapshot = RecoveryEngine.snapshot_with_conflicts(
        &store,
        &store,
        conversation_id,
        Some(turn_id),
        conflicts,
    );

    assert_eq!(snapshot.readiness, RecoveryReadiness::HasConflict);
    assert_eq!(snapshot.conflicts.len(), 1);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "recovery.conflict.detected"
            && diagnostic.severity == agent_types::RecoveryDiagnosticSeverity::Error
    }));
}
