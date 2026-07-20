use ruse_worktree::{ChangeKind, ConflictKind, FileChange, WorktreeError, detect_conflicts};

fn change(path: &str, kind: ChangeKind, hash: Option<&str>) -> FileChange {
    FileChange::new(path, kind, hash.map(str::to_string)).expect("valid file change")
}

#[test]
fn conflict_detection_reports_no_conflicts_for_distinct_paths() {
    let lead = vec![change("src/a.rs", ChangeKind::Modified, Some("hash-a"))];
    let subagent = vec![change("src/b.rs", ChangeKind::Modified, Some("hash-b"))];

    let report = detect_conflicts(&lead, &subagent).unwrap();

    assert!(report.conflicts.is_empty());
    assert_eq!(report.checked_paths, vec!["src/a.rs", "src/b.rs"]);
}

#[test]
fn conflict_detection_reports_both_modified() {
    let lead = vec![change("src/lib.rs", ChangeKind::Modified, Some("lead"))];
    let subagent = vec![change("src/lib.rs", ChangeKind::Modified, Some("subagent"))];

    let report = detect_conflicts(&lead, &subagent).unwrap();
    let value = serde_json::to_value(&report).expect("serialize conflict report");

    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].path, "src/lib.rs");
    assert_eq!(report.conflicts[0].kind, ConflictKind::BothModified);
    assert_eq!(value["conflicts"][0]["kind"], "both_modified");
    assert_eq!(value["conflicts"][0]["lead_change"]["content_hash"], "lead");
}

#[test]
fn conflict_detection_reports_delete_modify() {
    let lead = vec![change("src/lib.rs", ChangeKind::Deleted, None)];
    let subagent = vec![change("src/lib.rs", ChangeKind::Modified, Some("subagent"))];

    let report = detect_conflicts(&lead, &subagent).unwrap();

    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].kind, ConflictKind::DeleteModify);
}

#[test]
fn conflict_detection_allows_add_add_same_hash() {
    let lead = vec![change("src/new.rs", ChangeKind::Added, Some("same"))];
    let subagent = vec![change("src/new.rs", ChangeKind::Added, Some("same"))];

    let report = detect_conflicts(&lead, &subagent).unwrap();

    assert!(report.conflicts.is_empty());
    assert_eq!(report.checked_paths, vec!["src/new.rs"]);
}

#[test]
fn conflict_detection_reports_add_add_different_hash() {
    let lead = vec![change("src/new.rs", ChangeKind::Added, Some("lead"))];
    let subagent = vec![change("src/new.rs", ChangeKind::Added, Some("subagent"))];

    let report = detect_conflicts(&lead, &subagent).unwrap();

    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(
        report.conflicts[0].kind,
        ConflictKind::AddAddDifferentContent
    );
}

#[test]
fn conflict_detection_rejects_invalid_paths() {
    let empty = FileChange::new(" ", ChangeKind::Modified, None);
    let traversal = FileChange::new("../secret", ChangeKind::Modified, None);

    assert!(matches!(empty, Err(WorktreeError::InvalidRequest(_))));
    assert!(matches!(traversal, Err(WorktreeError::InvalidRequest(_))));
}
