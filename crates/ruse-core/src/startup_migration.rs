use std::collections::BTreeSet;
use std::fmt;

use ruse_storage::{MigrationRegistry, SchemaFamily, SchemaVersion};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchemaVersionRecord {
    pub domain: String,
    pub current_version: u32,
    pub target_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct StartupMigrationStep {
    pub domain: String,
    pub migration_id: String,
    pub from_version: u32,
    pub to_version: u32,
    pub backup_required: bool,
}

impl StartupMigrationStep {
    pub fn new(
        domain: impl Into<String>,
        migration_id: impl Into<String>,
        from_version: u32,
        to_version: u32,
        backup_required: bool,
    ) -> Result<Self, StartupMigrationError> {
        let step = Self {
            domain: normalize_required("migration domain", domain.into())?,
            migration_id: normalize_required("migration id", migration_id.into())?,
            from_version,
            to_version,
            backup_required,
        };
        if step.to_version < step.from_version {
            return Err(StartupMigrationError::InvalidStep {
                domain: step.domain,
                migration_id: step.migration_id,
                message: "to_version must be greater than or equal to from_version".to_string(),
            });
        }
        Ok(step)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartupMigrationPlan {
    pub from_versions: Vec<SchemaVersionRecord>,
    pub steps: Vec<StartupMigrationStep>,
    pub backup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartupMigrationAuditRecord {
    pub domain: String,
    pub migration_id: String,
    pub from_version: u32,
    pub to_version: u32,
    pub status: StartupMigrationAuditStatus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupMigrationAuditStatus {
    Applied,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartupMigrationDiagnostic {
    pub severity: StartupMigrationDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub domain: Option<String>,
    pub migration_id: Option<String>,
}

impl StartupMigrationDiagnostic {
    #[must_use]
    pub fn blocked(
        domain: impl Into<String>,
        migration_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: StartupMigrationDiagnosticSeverity::Blocked,
            code: "startup_migration.blocked".to_string(),
            message: redact_migration_text(&message.into()),
            domain: Some(domain.into()),
            migration_id: Some(migration_id.into()),
        }
    }

    #[must_use]
    pub fn info(domain: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: StartupMigrationDiagnosticSeverity::Info,
            code: "startup_migration.info".to_string(),
            message: redact_migration_text(&message.into()),
            domain: Some(domain.into()),
            migration_id: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupMigrationDiagnosticSeverity {
    Info,
    Warning,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartupMigrationOutcome {
    pub plan: StartupMigrationPlan,
    pub applied_steps: Vec<StartupMigrationStep>,
    pub audit_records: Vec<StartupMigrationAuditRecord>,
    pub diagnostics: Vec<StartupMigrationDiagnostic>,
    pub runtime_allowed: bool,
}

pub trait StartupMigrationParticipant: fmt::Debug {
    fn domain(&self) -> &'static str;
    fn current_version(&self) -> u32;
    fn target_version(&self) -> u32;
    fn plan(&self) -> Result<Vec<StartupMigrationStep>, StartupMigrationError>;
    fn apply(&self, step: &StartupMigrationStep) -> Result<(), StartupMigrationError>;
    fn verify(&self) -> Result<(), StartupMigrationError>;
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StartupMigrationError {
    #[error("invalid startup migration step {domain}/{migration_id}: {message}")]
    InvalidStep {
        domain: String,
        migration_id: String,
        message: String,
    },
    #[error("duplicate startup migration domain: {domain}")]
    DuplicateDomain { domain: String },
    #[error("startup migration step failed for {domain}/{migration_id}: {message}")]
    StepFailed {
        domain: String,
        migration_id: String,
        message: String,
    },
    #[error("startup migration verification failed for {domain}: {message}")]
    VerificationFailed { domain: String, message: String },
    #[error("startup migration planning failed for {domain}: {message}")]
    PlanningFailed { domain: String, message: String },
}

#[derive(Debug)]
pub struct StartupMigrationRunner {
    participants: Vec<Box<dyn StartupMigrationParticipant>>,
    completed_steps: BTreeSet<(String, String)>,
    last_outcome: Option<StartupMigrationOutcome>,
}

impl StartupMigrationRunner {
    pub fn new(participants: Vec<Box<dyn StartupMigrationParticipant>>) -> Self {
        Self {
            participants,
            completed_steps: BTreeSet::new(),
            last_outcome: None,
        }
    }

    #[must_use]
    pub fn with_default_participants() -> Self {
        Self::new(default_startup_migration_participants())
    }

    pub fn plan(&self) -> Result<StartupMigrationPlan, StartupMigrationError> {
        let mut seen = BTreeSet::new();
        let mut from_versions = Vec::with_capacity(self.participants.len());
        let mut steps = Vec::new();

        for participant in &self.participants {
            let domain = participant.domain();
            if !seen.insert(domain) {
                return Err(StartupMigrationError::DuplicateDomain {
                    domain: domain.to_string(),
                });
            }
            from_versions.push(SchemaVersionRecord {
                domain: domain.to_string(),
                current_version: participant.current_version(),
                target_version: participant.target_version(),
            });
            let participant_steps =
                participant
                    .plan()
                    .map_err(|error| StartupMigrationError::PlanningFailed {
                        domain: domain.to_string(),
                        message: error.to_string(),
                    })?;
            steps.extend(
                participant_steps
                    .into_iter()
                    .filter(|step| !self.completed_steps.contains(&step_key(step))),
            );
        }
        let backup_required = steps.iter().any(|step| step.backup_required);
        Ok(StartupMigrationPlan {
            from_versions,
            steps,
            backup_required,
        })
    }

    pub fn apply(&mut self) -> Result<StartupMigrationOutcome, StartupMigrationError> {
        let plan = self.plan()?;
        let mut outcome = StartupMigrationOutcome {
            plan: plan.clone(),
            applied_steps: Vec::new(),
            audit_records: Vec::new(),
            diagnostics: Vec::new(),
            runtime_allowed: false,
        };

        for step in &plan.steps {
            let Some(participant) = self
                .participants
                .iter()
                .find(|participant| participant.domain() == step.domain)
            else {
                let error = StartupMigrationError::StepFailed {
                    domain: step.domain.clone(),
                    migration_id: step.migration_id.clone(),
                    message: "participant disappeared before apply".to_string(),
                };
                outcome.diagnostics.push(error_diagnostic(&error));
                self.last_outcome = Some(outcome);
                return Err(error);
            };

            if let Err(error) = participant.apply(step) {
                outcome.diagnostics.push(error_diagnostic(&error));
                self.last_outcome = Some(outcome);
                return Err(error);
            }
            self.completed_steps.insert(step_key(step));
            outcome.applied_steps.push(step.clone());
            outcome.audit_records.push(StartupMigrationAuditRecord {
                domain: step.domain.clone(),
                migration_id: step.migration_id.clone(),
                from_version: step.from_version,
                to_version: step.to_version,
                status: StartupMigrationAuditStatus::Applied,
            });
        }

        for participant in &self.participants {
            if let Err(error) = participant.verify() {
                outcome.diagnostics.push(error_diagnostic(&error));
                self.last_outcome = Some(outcome);
                return Err(error);
            }
        }

        outcome.runtime_allowed = true;
        self.last_outcome = Some(outcome.clone());
        Ok(outcome)
    }

    #[must_use]
    pub fn last_outcome(&self) -> Option<&StartupMigrationOutcome> {
        self.last_outcome.as_ref()
    }
}

#[derive(Debug)]
pub struct NoopStartupMigrationParticipant {
    domain: &'static str,
    version: u32,
}

impl NoopStartupMigrationParticipant {
    #[must_use]
    pub const fn new(domain: &'static str, version: u32) -> Self {
        Self { domain, version }
    }
}

impl StartupMigrationParticipant for NoopStartupMigrationParticipant {
    fn domain(&self) -> &'static str {
        self.domain
    }

    fn current_version(&self) -> u32 {
        self.version
    }

    fn target_version(&self) -> u32 {
        self.version
    }

    fn plan(&self) -> Result<Vec<StartupMigrationStep>, StartupMigrationError> {
        Ok(Vec::new())
    }

    fn apply(&self, _step: &StartupMigrationStep) -> Result<(), StartupMigrationError> {
        Ok(())
    }

    fn verify(&self) -> Result<(), StartupMigrationError> {
        Ok(())
    }
}

pub struct StorageJsonStartupMigrationParticipant {
    domain: &'static str,
    family: SchemaFamily,
    current_version: u32,
    target_version: u32,
    registry: MigrationRegistry,
    input: Value,
}

impl fmt::Debug for StorageJsonStartupMigrationParticipant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StorageJsonStartupMigrationParticipant")
            .field("domain", &self.domain)
            .field("family", &self.family)
            .field("current_version", &self.current_version)
            .field("target_version", &self.target_version)
            .finish_non_exhaustive()
    }
}

impl StorageJsonStartupMigrationParticipant {
    #[must_use]
    pub fn new(
        domain: &'static str,
        family: SchemaFamily,
        current_version: u32,
        target_version: u32,
        registry: MigrationRegistry,
        input: Value,
    ) -> Self {
        Self {
            domain,
            family,
            current_version,
            target_version,
            registry,
            input,
        }
    }
}

impl StartupMigrationParticipant for StorageJsonStartupMigrationParticipant {
    fn domain(&self) -> &'static str {
        self.domain
    }

    fn current_version(&self) -> u32 {
        self.current_version
    }

    fn target_version(&self) -> u32 {
        self.target_version
    }

    fn plan(&self) -> Result<Vec<StartupMigrationStep>, StartupMigrationError> {
        if self.current_version >= self.target_version {
            return Ok(Vec::new());
        }
        Ok(vec![StartupMigrationStep::new(
            self.domain,
            format!(
                "{}.{:?}.v{}_to_v{}",
                self.domain, self.family, self.current_version, self.target_version
            )
            .to_ascii_lowercase(),
            self.current_version,
            self.target_version,
            true,
        )?])
    }

    fn apply(&self, step: &StartupMigrationStep) -> Result<(), StartupMigrationError> {
        let outcome = self
            .registry
            .migrate(
                self.family,
                self.input.clone(),
                SchemaVersion(step.from_version),
                SchemaVersion(step.to_version),
            )
            .map_err(|error| StartupMigrationError::StepFailed {
                domain: step.domain.clone(),
                migration_id: step.migration_id.clone(),
                message: error.to_string(),
            })?;
        if outcome.report.blocked {
            return Err(StartupMigrationError::StepFailed {
                domain: step.domain.clone(),
                migration_id: step.migration_id.clone(),
                message: "storage migration was blocked".to_string(),
            });
        }
        Ok(())
    }

    fn verify(&self) -> Result<(), StartupMigrationError> {
        Ok(())
    }
}

#[must_use]
pub fn default_startup_migration_participants() -> Vec<Box<dyn StartupMigrationParticipant>> {
    vec![
        Box::new(NoopStartupMigrationParticipant::new("config", 1)),
        Box::new(NoopStartupMigrationParticipant::new("event", 1)),
        Box::new(NoopStartupMigrationParticipant::new("checkpoint", 1)),
        Box::new(NoopStartupMigrationParticipant::new("memory", 1)),
        Box::new(NoopStartupMigrationParticipant::new("plugin", 1)),
        Box::new(NoopStartupMigrationParticipant::new("prompt", 1)),
    ]
}

#[must_use]
pub fn startup_migration_status_json(outcome: &StartupMigrationOutcome) -> Value {
    json!({
        "runtime_allowed": outcome.runtime_allowed,
        "step_count": outcome.plan.steps.len(),
        "applied_count": outcome.applied_steps.len(),
        "diagnostic_count": outcome.diagnostics.len(),
    })
}

fn step_key(step: &StartupMigrationStep) -> (String, String) {
    (step.domain.clone(), step.migration_id.clone())
}

fn error_diagnostic(error: &StartupMigrationError) -> StartupMigrationDiagnostic {
    match error {
        StartupMigrationError::InvalidStep {
            domain,
            migration_id,
            message,
        }
        | StartupMigrationError::StepFailed {
            domain,
            migration_id,
            message,
        } => StartupMigrationDiagnostic::blocked(domain, migration_id, message),
        StartupMigrationError::DuplicateDomain { domain }
        | StartupMigrationError::VerificationFailed { domain, .. }
        | StartupMigrationError::PlanningFailed { domain, .. } => {
            StartupMigrationDiagnostic::blocked(domain, "startup", error.to_string())
        }
    }
}

fn normalize_required(label: &str, value: String) -> Result<String, StartupMigrationError> {
    let value = value.trim();
    if value.is_empty() {
        Err(StartupMigrationError::InvalidStep {
            domain: label.to_string(),
            migration_id: label.to_string(),
            message: format!("{label} cannot be empty"),
        })
    } else {
        Ok(value.to_string())
    }
}

fn redact_migration_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("token")
                || lower.contains("api_key")
                || lower.contains("password")
                || lower.contains("secret")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
