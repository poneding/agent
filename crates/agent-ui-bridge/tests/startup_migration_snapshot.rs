use agent_core::StartupMigrationRunner;
use agent_ui_bridge::{UiJsonFrame, UiStartupMigrationSnapshot, encode_json_frame};

#[test]
fn startup_migration_snapshot_serializes_ui_safe_shape() {
    let mut runner = StartupMigrationRunner::with_default_participants();
    let outcome = runner.apply().expect("startup migration outcome");

    let snapshot = UiStartupMigrationSnapshot::from_outcome(&outcome).expect("snapshot");

    assert!(snapshot.runtime_allowed);
    assert_eq!(snapshot.step_count, 0);
    assert_eq!(snapshot.applied_count, 0);
    assert_eq!(snapshot.diagnostic_count, 0);
    assert_eq!(
        snapshot.domains,
        vec![
            "checkpoint",
            "config",
            "event",
            "memory",
            "plugin",
            "prompt"
        ]
    );

    let line = encode_json_frame(&UiJsonFrame::startup_migration_snapshot(snapshot)).expect("frame");
    assert!(line.contains("\"type\":\"startup_migration_snapshot\""));
    assert_eq!(line.matches('\n').count(), 1);
}
