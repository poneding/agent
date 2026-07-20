use std::path::{Path, PathBuf};

use ruse_skill::{ManagedSkillRootRegistry, SkillRootError};

fn write_skill(root: &Path, name: &str, description: &str, body: &str) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(
        root.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: {description}\n---\n{body}\n"),
    )
    .unwrap();
}

#[test]
fn skill_roots_add_list_enable_disable_remove_with_deterministic_ids() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("code-review");
    write_skill(
        &root,
        "Code Review",
        "Review code changes",
        "Check diffs carefully.",
    );

    let mut registry = ManagedSkillRootRegistry::default();
    let added = registry.add(root.clone(), true).unwrap();
    let added_again = ManagedSkillRootRegistry::default()
        .add(root.clone(), true)
        .unwrap();

    assert_eq!(added.root_id, added_again.root_id);
    assert!(added.root_id.starts_with("skill-root-"));
    assert_eq!(added.path, root);
    assert!(added.enabled);

    assert_eq!(registry.list(), vec![added.clone()]);
    assert!(!registry.disable(&added.root_id).unwrap().enabled);
    assert!(!registry.inspect(&added.root_id).unwrap().root.enabled);
    assert!(registry.enable(&added.root_id).unwrap().enabled);

    let removed = registry.remove(&added.root_id).unwrap();
    assert_eq!(removed.root_id, added.root_id);
    assert!(registry.list().is_empty());
}

#[test]
fn skill_roots_reject_missing_duplicate_and_unknown_roots() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("skill");
    write_skill(&root, "Test Skill", "Test skill", "Body");

    let mut registry = ManagedSkillRootRegistry::default();
    let added = registry.add(root.clone(), false).unwrap();

    let duplicate = registry.add(root, true).unwrap_err();
    assert!(matches!(duplicate, SkillRootError::DuplicatePath(_)));

    let missing = registry.add(temp.path().join("missing"), true).unwrap_err();
    assert!(matches!(missing, SkillRootError::MissingPath(_)));

    let unknown = registry.inspect("skill-root-unknown").unwrap_err();
    assert_eq!(
        unknown,
        SkillRootError::UnknownRoot("skill-root-unknown".to_string())
    );
    assert_eq!(registry.list(), vec![added]);
}

#[test]
fn skill_roots_inspect_valid_root_without_exposing_prompt_body() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("code-review");
    write_skill(
        &root,
        "Code Review",
        "Review code changes",
        "Secret-free body",
    );

    let mut registry = ManagedSkillRootRegistry::default();
    let added = registry.add(root, true).unwrap();
    let inspection = registry.inspect(&added.root_id).unwrap();

    assert!(inspection.diagnostics.is_empty());
    assert_eq!(inspection.discovered.len(), 1);
    assert_eq!(inspection.discovered[0].skill_id, "code-review");
    assert_eq!(inspection.discovered[0].name, "Code Review");
    assert!(inspection.discovered[0].source_hash.is_some());
    assert!(!format!("{inspection:?}").contains("Secret-free body"));
}

#[test]
fn skill_roots_report_diagnostics_for_invalid_and_disabled_roots() {
    let temp = tempfile::tempdir().unwrap();
    let missing_manifest_root = temp.path().join("missing-manifest");
    std::fs::create_dir_all(&missing_manifest_root).unwrap();
    let empty_root = temp.path().join("empty-skill");
    write_skill(&empty_root, "Empty Skill", "Empty prompt", "   ");
    let disabled_root = temp.path().join("disabled");
    write_skill(
        &disabled_root,
        "Disabled Skill",
        "Disabled prompt",
        "Disabled body",
    );

    let mut registry = ManagedSkillRootRegistry::default();
    let missing_manifest = registry.add(missing_manifest_root, true).unwrap();
    let empty = registry.add(empty_root, true).unwrap();
    let disabled = registry.add(disabled_root, false).unwrap();

    let missing_inspection = registry.inspect(&missing_manifest.root_id).unwrap();
    assert_eq!(missing_inspection.diagnostics[0].code, "manifest_not_found");
    assert!(missing_inspection.discovered.is_empty());

    let empty_inspection = registry.inspect(&empty.root_id).unwrap();
    assert_eq!(empty_inspection.diagnostics[0].code, "invalid_prompt");
    assert!(empty_inspection.discovered.is_empty());

    let disabled_inspection = registry.inspect(&disabled.root_id).unwrap();
    assert_eq!(disabled_inspection.diagnostics[0].code, "root_disabled");
    assert!(disabled_inspection.discovered.is_empty());
}

#[test]
fn skill_roots_detect_duplicate_skill_ids_across_enabled_roots() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    write_skill(&first, "Same Skill", "First", "First body");
    write_skill(&second, "Same Skill", "Second", "Second body");

    let mut registry = ManagedSkillRootRegistry::default();
    registry.add(first, true).unwrap();
    registry.add(second, true).unwrap();

    let inspections = registry.inspect_all();
    let duplicate_count = inspections
        .iter()
        .flat_map(|inspection| &inspection.diagnostics)
        .filter(|diagnostic| diagnostic.code == "duplicate_skill_id")
        .count();

    assert_eq!(duplicate_count, 2);
}

#[test]
fn skill_roots_redact_secret_like_diagnostics() {
    let mut registry = ManagedSkillRootRegistry::default();
    let path = PathBuf::from("/tmp/sk-secret-token");

    let error = registry.add(path, true).unwrap_err();

    assert!(!error.to_string().contains("sk-secret-token"));
    assert!(!error.to_string().contains("token"));
    assert!(error.to_string().contains("[redacted]"));
}
