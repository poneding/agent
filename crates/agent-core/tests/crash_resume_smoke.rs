use agent_core::run_recovery_crash_smoke_fixture;
use agent_types::{CommandRetryReadiness, RecoveryReadiness};

#[test]
fn crash_restart_recovery_smoke_preserves_safe_recovery_evidence() {
    let report = run_recovery_crash_smoke_fixture().expect("recovery crash smoke");

    assert!(report.snapshot.checkpoint.is_some());
    assert_eq!(report.snapshot.replay.replay_from.get(), 2);
    assert_eq!(report.snapshot.replay.replayed_event_count, 2);
    assert_eq!(report.snapshot.pending_tool_calls.len(), 1);
    assert_eq!(
        report.snapshot.pending_tool_calls[0].call_id,
        "call-pending"
    );
    assert_eq!(report.snapshot.idempotent_effects.len(), 1);
    assert_eq!(report.snapshot.idempotent_recovery.reusable_effect_count, 1);
    assert_eq!(report.snapshot.idempotent_recovery.pending_tool_count, 1);
    assert_eq!(report.snapshot.conflicts.len(), 1);
    assert_eq!(report.snapshot.conflicts[0].path, "src/[redacted]");
    assert_eq!(report.snapshot.readiness, RecoveryReadiness::HasConflict);

    assert!(report.dry_run.blocked);
    assert_eq!(report.dry_run.will_replay_event_count, 2);
    assert_eq!(report.dry_run.will_resume_pending_tool_count, 1);
    assert_eq!(report.dry_run.will_reuse_idempotent_effect_count, 1);
    assert_eq!(
        report.dry_run.snapshot.readiness,
        RecoveryReadiness::HasConflict
    );

    assert_eq!(
        report.retry_plan.readiness,
        CommandRetryReadiness::Retryable
    );
    assert!(report.retry_plan.dry_run);
    assert_eq!(report.retry_plan.exit_code, Some(101));
    assert!(
        report
            .retry_plan
            .attempt
            .next_attempt_id
            .ends_with("-retry-1")
    );

    let serialized = serde_json::to_string(&report).expect("serialize evidence");
    assert!(serialized.contains("[redacted]"));
    for forbidden in ["token-secret", "api_key", "password"] {
        assert!(
            !serialized.contains(forbidden),
            "serialized recovery smoke leaked {forbidden}: {serialized}"
        );
    }
}
