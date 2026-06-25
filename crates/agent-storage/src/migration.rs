use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFamily {
    Event,
    Checkpoint,
    Config,
    Memory,
    Plugin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub u32);

pub trait SchemaMigration: Send + Sync {
    fn family(&self) -> SchemaFamily;
    #[allow(clippy::wrong_self_convention)]
    fn from_version(&self) -> SchemaVersion;
    fn to_version(&self) -> SchemaVersion;
    fn id(&self) -> &str;
    fn apply(&self, input: Value) -> Result<Value, MigrationError>;

    fn diagnostics(&self, _before: &Value, _after: &Value) -> Vec<MigrationDiagnostic> {
        Vec::new()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MigrationOutcome {
    pub output: Value,
    pub report: MigrationReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MigrationReport {
    pub family: SchemaFamily,
    pub from_version: SchemaVersion,
    pub to_version: SchemaVersion,
    pub applied_migrations: Vec<String>,
    pub skipped_migrations: Vec<String>,
    pub diagnostics: Vec<MigrationDiagnostic>,
    pub blocked: bool,
}

impl MigrationReport {
    fn blocked_missing_step(
        family: SchemaFamily,
        from_version: SchemaVersion,
        to_version: SchemaVersion,
        missing_from_version: SchemaVersion,
        missing_to_version: SchemaVersion,
    ) -> Self {
        Self {
            family,
            from_version,
            to_version,
            applied_migrations: Vec::new(),
            skipped_migrations: Vec::new(),
            diagnostics: vec![MigrationDiagnostic::blocked_missing_step(
                missing_from_version,
                missing_to_version,
            )],
            blocked: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MigrationDiagnostic {
    pub severity: MigrationDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

impl MigrationDiagnostic {
    #[must_use]
    pub fn blocked_missing_step(from_version: SchemaVersion, to_version: SchemaVersion) -> Self {
        Self {
            severity: MigrationDiagnosticSeverity::Blocked,
            code: "migration.missing_step".to_string(),
            message: format!(
                "missing migration step from version {} to {}",
                from_version.0, to_version.0
            ),
            field: Some("schema_version".to_string()),
        }
    }

    #[must_use]
    pub fn permission_broadening<I, S>(permissions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let permissions = permissions
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .join(", ");
        Self {
            severity: MigrationDiagnosticSeverity::Warning,
            code: "config.permission_broadened".to_string(),
            message: format!("config migration broadens permissions: {permissions}"),
            field: Some("permissions".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationDiagnosticSeverity {
    Info,
    Warning,
    Blocked,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum MigrationError {
    #[error("duplicate migration id {migration_id}")]
    DuplicateMigrationId { migration_id: String },
    #[error("duplicate migration step for {family:?} {from_version:?} -> {to_version:?}")]
    DuplicateVersionPair {
        family: SchemaFamily,
        from_version: SchemaVersion,
        to_version: SchemaVersion,
    },
    #[error("migration {migration_id} must advance exactly one schema version")]
    NonContiguousMigration { migration_id: String },
    #[error("invalid migration range for {family:?}: {from_version:?} -> {to_version:?}")]
    InvalidVersionRange {
        family: SchemaFamily,
        from_version: SchemaVersion,
        to_version: SchemaVersion,
    },
    #[error("migration fixture json is invalid: {0}")]
    Json(String),
    #[error("migration {migration_id} failed: {message}")]
    ApplyFailed {
        migration_id: String,
        message: String,
    },
    #[error(
        "config migration {migration_id} broadens permissions without diagnostic: {permissions:?}"
    )]
    PermissionBroadeningRequiresDiagnostic {
        migration_id: String,
        permissions: Vec<String>,
    },
}

#[derive(Default)]
pub struct MigrationRegistry {
    migrations: Vec<Box<dyn SchemaMigration>>,
}

impl MigrationRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<M>(&mut self, migration: M) -> Result<(), MigrationError>
    where
        M: SchemaMigration + 'static,
    {
        let family = migration.family();
        let from_version = migration.from_version();
        let to_version = migration.to_version();
        let migration_id = migration.id().to_string();

        if to_version.0 != from_version.0 + 1 {
            return Err(MigrationError::NonContiguousMigration { migration_id });
        }
        if self
            .migrations
            .iter()
            .any(|existing| existing.id() == migration.id())
        {
            return Err(MigrationError::DuplicateMigrationId { migration_id });
        }
        if self.migrations.iter().any(|existing| {
            existing.family() == family
                && existing.from_version() == from_version
                && existing.to_version() == to_version
        }) {
            return Err(MigrationError::DuplicateVersionPair {
                family,
                from_version,
                to_version,
            });
        }

        self.migrations.push(Box::new(migration));
        Ok(())
    }

    pub fn migrate_json_str(
        &self,
        family: SchemaFamily,
        input: &str,
        from_version: SchemaVersion,
        to_version: SchemaVersion,
    ) -> Result<MigrationOutcome, MigrationError> {
        let input =
            serde_json::from_str(input).map_err(|error| MigrationError::Json(error.to_string()))?;
        self.migrate(family, input, from_version, to_version)
    }

    pub fn migrate(
        &self,
        family: SchemaFamily,
        input: Value,
        from_version: SchemaVersion,
        to_version: SchemaVersion,
    ) -> Result<MigrationOutcome, MigrationError> {
        if from_version > to_version {
            return Err(MigrationError::InvalidVersionRange {
                family,
                from_version,
                to_version,
            });
        }

        let mut path = Vec::new();
        let mut current = from_version;
        while current < to_version {
            let next = SchemaVersion(current.0 + 1);
            let Some(migration) = self.find_step(family, current, next) else {
                return Ok(MigrationOutcome {
                    output: input,
                    report: MigrationReport::blocked_missing_step(
                        family,
                        from_version,
                        to_version,
                        current,
                        next,
                    ),
                });
            };
            path.push(migration);
            current = next;
        }

        let mut output = input;
        let mut applied_migrations = Vec::new();
        let mut diagnostics = Vec::new();

        for migration in path {
            let before = output.clone();
            let migration_id = migration.id().to_string();
            let after = migration
                .apply(output)
                .map_err(|error| MigrationError::ApplyFailed {
                    migration_id: migration_id.clone(),
                    message: error.to_string(),
                })?;
            let step_diagnostics = migration.diagnostics(&before, &after);

            if family == SchemaFamily::Config {
                reject_silent_permission_broadening(
                    &migration_id,
                    &before,
                    &after,
                    &step_diagnostics,
                )?;
            }

            diagnostics.extend(step_diagnostics);
            applied_migrations.push(migration_id);
            output = after;
        }

        Ok(MigrationOutcome {
            output,
            report: MigrationReport {
                family,
                from_version,
                to_version,
                applied_migrations,
                skipped_migrations: Vec::new(),
                diagnostics,
                blocked: false,
            },
        })
    }

    fn find_step(
        &self,
        family: SchemaFamily,
        from_version: SchemaVersion,
        to_version: SchemaVersion,
    ) -> Option<&dyn SchemaMigration> {
        self.migrations.iter().map(Box::as_ref).find(|migration| {
            migration.family() == family
                && migration.from_version() == from_version
                && migration.to_version() == to_version
        })
    }
}

fn reject_silent_permission_broadening(
    migration_id: &str,
    before: &Value,
    after: &Value,
    diagnostics: &[MigrationDiagnostic],
) -> Result<(), MigrationError> {
    let broadened_permissions = broadened_permissions(before, after);
    if broadened_permissions.is_empty()
        || diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "config.permission_broadened")
    {
        return Ok(());
    }

    Err(MigrationError::PermissionBroadeningRequiresDiagnostic {
        migration_id: migration_id.to_string(),
        permissions: broadened_permissions,
    })
}

fn broadened_permissions(before: &Value, after: &Value) -> Vec<String> {
    let before = permission_set(before);
    let after = permission_set(after);
    after.difference(&before).cloned().collect()
}

fn permission_set(value: &Value) -> BTreeSet<String> {
    value
        .get("permissions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}
