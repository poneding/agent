# M12-01 Schema Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add schema migration contracts, registry behavior, diagnostics, config permission-broadening checks, and fixture coverage in `agent-storage`.

**Architecture:** Keep migration contracts in `agent-storage`; do not make storage depend on core, UI, model, MCP, or plugin crates. Migrations operate on `serde_json::Value` in M12 so durable schema behavior can be tested before backend-specific migrations are added.

**Tech Stack:** Rust 2024 edition, existing `agent-storage` crate, serde, serde_json, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M12-01-schema-migration.spec.md`
- Read: `docs/specs/M12-00-production-hardening-technical-design.spec.md`
- Read: `crates/agent-storage/src/lib.rs`
- Read: `crates/agent-storage/Cargo.toml`

- [x] **Step 1: Review migration scope**

Run:

```bash
sed -n '1,260p' docs/specs/M12-01-schema-migration.spec.md
sed -n '1,220p' docs/specs/M12-00-production-hardening-technical-design.spec.md
sed -n '1,180p' crates/agent-storage/src/lib.rs
sed -n '1,120p' crates/agent-storage/Cargo.toml
```

Expected: migration ownership stays in `agent-storage`, and `serde`, `serde_json`, and `thiserror` already exist as dependencies.

### Task 2: Write RED Migration Tests

**Files:**
- Create: `crates/agent-storage/tests/schema_migration.rs`

- [x] **Step 1: Add failing tests**

Create `crates/agent-storage/tests/schema_migration.rs` with this content:

```rust
use agent_storage::{
    MigrationDiagnostic, MigrationDiagnosticSeverity, MigrationError, MigrationRegistry,
    SchemaFamily, SchemaMigration, SchemaVersion,
};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
struct StepMigration {
    id: &'static str,
    family: SchemaFamily,
    from: SchemaVersion,
    to: SchemaVersion,
}

impl StepMigration {
    const fn new(
        id: &'static str,
        family: SchemaFamily,
        from: u32,
        to: u32,
    ) -> Self {
        Self {
            id,
            family,
            from: SchemaVersion(from),
            to: SchemaVersion(to),
        }
    }
}

impl SchemaMigration for StepMigration {
    fn family(&self) -> SchemaFamily {
        self.family
    }

    fn from_version(&self) -> SchemaVersion {
        self.from
    }

    fn to_version(&self) -> SchemaVersion {
        self.to
    }

    fn id(&self) -> &str {
        self.id
    }

    fn apply(&self, mut input: Value) -> Result<Value, MigrationError> {
        let object = input.as_object_mut().expect("fixture object");
        object.insert("schema_version".to_string(), json!(self.to.0));
        object
            .entry("steps")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .expect("steps array")
            .push(json!(self.id));
        Ok(input)
    }
}

#[test]
fn schema_migration_registry_orders_contiguous_steps() {
    let mut registry = MigrationRegistry::new();
    registry
        .register(StepMigration::new("event.v2_to_v3", SchemaFamily::Event, 2, 3))
        .expect("register second step first");
    registry
        .register(StepMigration::new("event.v1_to_v2", SchemaFamily::Event, 1, 2))
        .expect("register first step second");

    let outcome = registry
        .migrate(
            SchemaFamily::Event,
            json!({ "schema_version": 1, "steps": [] }),
            SchemaVersion(1),
            SchemaVersion(3),
        )
        .expect("migration succeeds");

    assert_eq!(outcome.output["schema_version"], json!(3));
    assert_eq!(
        outcome.output["steps"],
        json!(["event.v1_to_v2", "event.v2_to_v3"])
    );
    assert_eq!(
        outcome.report.applied_migrations,
        vec!["event.v1_to_v2".to_string(), "event.v2_to_v3".to_string()]
    );
    assert!(!outcome.report.blocked);
}

#[test]
fn schema_migration_registry_rejects_duplicate_step() {
    let mut registry = MigrationRegistry::new();
    registry
        .register(StepMigration::new("event.v1_to_v2", SchemaFamily::Event, 1, 2))
        .expect("first registration succeeds");

    let error = registry
        .register(StepMigration::new("event.copy_v1_to_v2", SchemaFamily::Event, 1, 2))
        .expect_err("duplicate version pair rejected");

    assert!(matches!(
        error,
        MigrationError::DuplicateVersionPair {
            family: SchemaFamily::Event,
            from_version: SchemaVersion(1),
            to_version: SchemaVersion(2)
        }
    ));
}

#[test]
fn schema_migration_registry_rejects_missing_step() {
    let mut registry = MigrationRegistry::new();
    registry
        .register(StepMigration::new("event.v1_to_v2", SchemaFamily::Event, 1, 2))
        .expect("register first step");

    let outcome = registry
        .migrate(
            SchemaFamily::Event,
            json!({ "schema_version": 1, "steps": [] }),
            SchemaVersion(1),
            SchemaVersion(3),
        )
        .expect("missing step becomes blocked report");

    assert_eq!(outcome.output, json!({ "schema_version": 1, "steps": [] }));
    assert!(outcome.report.blocked);
    assert!(outcome.report.applied_migrations.is_empty());
    assert_eq!(outcome.report.diagnostics[0].severity, MigrationDiagnosticSeverity::Blocked);
    assert_eq!(outcome.report.diagnostics[0].code, "migration.missing_step");
}

#[test]
fn schema_migration_report_serializes_stable_shape() {
    let report = agent_storage::MigrationReport {
        family: SchemaFamily::Event,
        from_version: SchemaVersion(1),
        to_version: SchemaVersion(2),
        applied_migrations: vec!["event.v1_to_v2".to_string()],
        skipped_migrations: Vec::new(),
        diagnostics: vec![MigrationDiagnostic {
            severity: MigrationDiagnosticSeverity::Warning,
            code: "fixture.warning".to_string(),
            message: "fixture warning".to_string(),
            field: Some("schema_version".to_string()),
        }],
        blocked: false,
    };

    assert_eq!(
        serde_json::to_value(report).expect("serialize report"),
        json!({
            "family": "event",
            "from_version": 1,
            "to_version": 2,
            "applied_migrations": ["event.v1_to_v2"],
            "skipped_migrations": [],
            "diagnostics": [{
                "severity": "warning",
                "code": "fixture.warning",
                "message": "fixture warning",
                "field": "schema_version"
            }],
            "blocked": false
        })
    );
}

#[test]
fn schema_migration_invalid_fixture_json_returns_recoverable_error() {
    let registry = MigrationRegistry::new();

    let error = registry
        .migrate_json_str(
            SchemaFamily::Event,
            "{not-json",
            SchemaVersion(1),
            SchemaVersion(2),
        )
        .expect_err("invalid fixture json is recoverable");

    assert!(matches!(error, MigrationError::Json(message) if !message.is_empty()));
}

#[derive(Clone, Copy, Debug)]
struct ConfigPermissionMigration {
    emits_diagnostic: bool,
}

impl SchemaMigration for ConfigPermissionMigration {
    fn family(&self) -> SchemaFamily {
        SchemaFamily::Config
    }

    fn from_version(&self) -> SchemaVersion {
        SchemaVersion(1)
    }

    fn to_version(&self) -> SchemaVersion {
        SchemaVersion(2)
    }

    fn id(&self) -> &str {
        "config.v1_to_v2"
    }

    fn apply(&self, mut input: Value) -> Result<Value, MigrationError> {
        let object = input.as_object_mut().expect("fixture object");
        object.insert("schema_version".to_string(), json!(2));
        object.insert("permissions".to_string(), json!(["file.read", "shell.exec"]));
        Ok(input)
    }

    fn diagnostics(&self, _before: &Value, _after: &Value) -> Vec<MigrationDiagnostic> {
        if self.emits_diagnostic {
            vec![MigrationDiagnostic::permission_broadening(["shell.exec"])]
        } else {
            Vec::new()
        }
    }
}

#[test]
fn schema_migration_config_broadening_requires_diagnostic() {
    let mut registry = MigrationRegistry::new();
    registry
        .register(ConfigPermissionMigration {
            emits_diagnostic: false,
        })
        .expect("register config migration");

    let error = registry
        .migrate(
            SchemaFamily::Config,
            json!({ "schema_version": 1, "permissions": ["file.read"] }),
            SchemaVersion(1),
            SchemaVersion(2),
        )
        .expect_err("permission broadening without diagnostic rejected");

    assert!(matches!(
        error,
        MigrationError::PermissionBroadeningRequiresDiagnostic {
            migration_id,
            permissions
        } if migration_id == "config.v1_to_v2" && permissions == vec!["shell.exec".to_string()]
    ));

    let mut registry = MigrationRegistry::new();
    registry
        .register(ConfigPermissionMigration {
            emits_diagnostic: true,
        })
        .expect("register diagnostic config migration");

    let outcome = registry
        .migrate(
            SchemaFamily::Config,
            json!({ "schema_version": 1, "permissions": ["file.read"] }),
            SchemaVersion(1),
            SchemaVersion(2),
        )
        .expect("explicit diagnostic allows migration");

    assert_eq!(outcome.report.diagnostics[0].code, "config.permission_broadened");
}

#[derive(Clone, Copy, Debug)]
struct EventPayloadMigration;

impl SchemaMigration for EventPayloadMigration {
    fn family(&self) -> SchemaFamily {
        SchemaFamily::Event
    }

    fn from_version(&self) -> SchemaVersion {
        SchemaVersion(1)
    }

    fn to_version(&self) -> SchemaVersion {
        SchemaVersion(2)
    }

    fn id(&self) -> &str {
        "event.payload_kind_to_type"
    }

    fn apply(&self, mut input: Value) -> Result<Value, MigrationError> {
        let object = input.as_object_mut().expect("fixture object");
        object.insert("schema_version".to_string(), json!(2));
        let payload = object
            .get_mut("payload")
            .and_then(Value::as_object_mut)
            .expect("payload object");
        let kind = payload.remove("kind").expect("kind field");
        payload.insert("event_type".to_string(), kind);
        Ok(input)
    }
}

#[test]
fn schema_migration_fixture_upgrades_event_payload() {
    let mut registry = MigrationRegistry::new();
    registry
        .register(EventPayloadMigration)
        .expect("register event payload migration");

    let outcome = registry
        .migrate(
            SchemaFamily::Event,
            json!({
                "schema_version": 1,
                "payload": {
                    "kind": "turn_started",
                    "message": "hello"
                }
            }),
            SchemaVersion(1),
            SchemaVersion(2),
        )
        .expect("event fixture migrates");

    assert_eq!(
        outcome.output,
        json!({
            "schema_version": 2,
            "payload": {
                "event_type": "turn_started",
                "message": "hello"
            }
        })
    );
    assert_eq!(
        outcome.report.applied_migrations,
        vec!["event.payload_kind_to_type".to_string()]
    );
}
```

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-storage schema_migration
```

Expected: compile failure mentioning missing migration types such as `MigrationRegistry`, `SchemaMigration`, or `SchemaVersion`.

### Task 3: Implement Migration Contracts

**Files:**
- Modify: `crates/agent-storage/src/lib.rs`
- Create: `crates/agent-storage/src/migration.rs`

- [x] **Step 1: Add module exports**

Modify `crates/agent-storage/src/lib.rs`:

```rust
pub mod migration;

pub use migration::{
    MigrationDiagnostic, MigrationDiagnosticSeverity, MigrationError, MigrationOutcome,
    MigrationRegistry, MigrationReport, SchemaFamily, SchemaMigration, SchemaVersion,
};
```

- [x] **Step 2: Add public migration types**

Create `crates/agent-storage/src/migration.rs` with these public contracts:

```rust
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
    fn from_version(&self) -> SchemaVersion;
    fn to_version(&self) -> SchemaVersion;
    fn id(&self) -> &str;
    fn apply(&self, input: Value) -> Result<Value, MigrationError>;

    fn diagnostics(&self, _before: &Value, _after: &Value) -> Vec<MigrationDiagnostic> {
        Vec::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MigrationDiagnostic {
    pub severity: MigrationDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    #[error("config migration {migration_id} broadens permissions without diagnostic: {permissions:?}")]
    PermissionBroadeningRequiresDiagnostic {
        migration_id: String,
        permissions: Vec<String>,
    },
}
```

- [x] **Step 3: Add registry execution**

Implement `MigrationRegistry` in `migration.rs`:

```rust
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
        let input = serde_json::from_str(input)
            .map_err(|error| MigrationError::Json(error.to_string()))?;
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
            let after = migration.apply(output).map_err(|error| MigrationError::ApplyFailed {
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
}
```

- [x] **Step 4: Add diagnostics and helper functions**

Complete `migration.rs` with:

```rust
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
```

Also add private helpers for `MigrationReport::blocked_missing_step`, `MigrationRegistry::find_step`, permission set extraction, and silent permission-broadening rejection. Permission extraction only needs to inspect a top-level JSON array named `permissions` in M12-01.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/agent-storage/tests/schema_migration.rs`
- Read: `crates/agent-storage/src/migration.rs`
- Read: `crates/agent-storage/src/lib.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-storage schema_migration
```

Expected: all seven schema migration tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-01-schema-migration.plan.md`
- Modify: `docs/plans/M12-production-hardening.plan.md`

- [x] **Step 1: Mark M12-01 complete and advance active focus**

Update:

```text
README.md
docs/milestones/README.md
docs/plans/M12-01-schema-migration.plan.md
docs/plans/M12-production-hardening.plan.md
```

Expected status:

```text
M12-01 implementation: Complete
Current focus: M12-02 Telemetry
```

- [x] **Step 2: Commit**

Run:

```bash
git add crates/agent-storage docs README.md AGENTS.md
git commit -m "feat: add m12 schema migration"
```

Expected: commit succeeds.
