use std::cell::Cell;

use ruse_core::{
    StartupMigrationError, StartupMigrationParticipant, StartupMigrationRunner,
    StartupMigrationStep,
};

#[test]
fn startup_migrations_plan_and_apply_ordered_steps_with_audit() {
    let mut runner = StartupMigrationRunner::new(vec![
        Box::new(FixtureParticipant::new("config", 1, 2)),
        Box::new(FixtureParticipant::new("storage", 2, 3)),
    ]);

    let plan = runner.plan().expect("plan");
    assert_eq!(plan.from_versions.len(), 2);
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| step.migration_id.as_str())
            .collect::<Vec<_>>(),
        vec!["config.v1_to_v2", "storage.v2_to_v3"]
    );
    assert!(plan.backup_required);

    let outcome = runner.apply().expect("apply");

    assert!(outcome.runtime_allowed);
    assert_eq!(outcome.applied_steps.len(), 2);
    assert_eq!(outcome.audit_records.len(), 2);
    assert_eq!(outcome.audit_records[0].domain, "config");
    assert_eq!(outcome.audit_records[1].domain, "storage");
}

#[test]
fn startup_migrations_repeat_apply_is_idempotent() {
    let mut runner =
        StartupMigrationRunner::new(vec![Box::new(FixtureParticipant::new("memory", 1, 2))]);

    let first = runner.apply().expect("first apply");
    let second = runner.apply().expect("second apply");

    assert_eq!(first.applied_steps.len(), 1);
    assert!(second.applied_steps.is_empty());
    assert!(second.audit_records.is_empty());
    assert!(second.runtime_allowed);
}

#[test]
fn startup_migrations_block_runtime_on_apply_failure() {
    let mut runner = StartupMigrationRunner::new(vec![
        Box::new(FixtureParticipant::new("config", 1, 2)),
        Box::new(FailingParticipant::new("plugin")),
    ]);

    let error = runner.apply().expect_err("apply failure blocks runtime");

    assert!(matches!(
        error,
        StartupMigrationError::StepFailed { ref domain, .. } if domain == "plugin"
    ));
    assert!(!runner.last_outcome().expect("last outcome").runtime_allowed);
}

#[test]
fn startup_migrations_default_participants_are_safe_noops() {
    let mut runner = StartupMigrationRunner::with_default_participants();

    let outcome = runner.apply().expect("default startup migration");

    assert!(outcome.runtime_allowed);
    assert!(outcome.applied_steps.is_empty());
    assert_eq!(
        outcome
            .plan
            .from_versions
            .iter()
            .map(|record| record.domain.as_str())
            .collect::<Vec<_>>(),
        vec![
            "config",
            "event",
            "checkpoint",
            "memory",
            "plugin",
            "prompt"
        ]
    );
}

#[derive(Debug)]
struct FixtureParticipant {
    domain: &'static str,
    current: Cell<u32>,
    target: u32,
}

impl FixtureParticipant {
    fn new(domain: &'static str, current: u32, target: u32) -> Self {
        Self {
            domain,
            current: Cell::new(current),
            target,
        }
    }
}

impl StartupMigrationParticipant for FixtureParticipant {
    fn domain(&self) -> &'static str {
        self.domain
    }

    fn current_version(&self) -> u32 {
        self.current.get()
    }

    fn target_version(&self) -> u32 {
        self.target
    }

    fn plan(&self) -> Result<Vec<StartupMigrationStep>, StartupMigrationError> {
        if self.current.get() >= self.target {
            return Ok(Vec::new());
        }
        Ok(vec![StartupMigrationStep::new(
            self.domain,
            format!(
                "{}.v{}_to_v{}",
                self.domain,
                self.current.get(),
                self.target
            ),
            self.current.get(),
            self.target,
            true,
        )?])
    }

    fn apply(&self, step: &StartupMigrationStep) -> Result<(), StartupMigrationError> {
        self.current.set(step.to_version);
        Ok(())
    }

    fn verify(&self) -> Result<(), StartupMigrationError> {
        if self.current.get() == self.target {
            Ok(())
        } else {
            Err(StartupMigrationError::VerificationFailed {
                domain: self.domain.to_string(),
                message: "target version was not reached".to_string(),
            })
        }
    }
}

#[derive(Debug)]
struct FailingParticipant {
    domain: &'static str,
}

impl FailingParticipant {
    const fn new(domain: &'static str) -> Self {
        Self { domain }
    }
}

impl StartupMigrationParticipant for FailingParticipant {
    fn domain(&self) -> &'static str {
        self.domain
    }

    fn current_version(&self) -> u32 {
        1
    }

    fn target_version(&self) -> u32 {
        2
    }

    fn plan(&self) -> Result<Vec<StartupMigrationStep>, StartupMigrationError> {
        Ok(vec![StartupMigrationStep::new(
            self.domain,
            "plugin.v1_to_v2",
            1,
            2,
            true,
        )?])
    }

    fn apply(&self, _step: &StartupMigrationStep) -> Result<(), StartupMigrationError> {
        Err(StartupMigrationError::StepFailed {
            domain: self.domain.to_string(),
            migration_id: "plugin.v1_to_v2".to_string(),
            message: "fixture failure".to_string(),
        })
    }

    fn verify(&self) -> Result<(), StartupMigrationError> {
        Ok(())
    }
}
