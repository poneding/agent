use ruse_storage::{
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
    const fn new(id: &'static str, family: SchemaFamily, from: u32, to: u32) -> Self {
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
        .register(StepMigration::new(
            "event.v2_to_v3",
            SchemaFamily::Event,
            2,
            3,
        ))
        .expect("register second step first");
    registry
        .register(StepMigration::new(
            "event.v1_to_v2",
            SchemaFamily::Event,
            1,
            2,
        ))
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
        .register(StepMigration::new(
            "event.v1_to_v2",
            SchemaFamily::Event,
            1,
            2,
        ))
        .expect("first registration succeeds");

    let error = registry
        .register(StepMigration::new(
            "event.copy_v1_to_v2",
            SchemaFamily::Event,
            1,
            2,
        ))
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
        .register(StepMigration::new(
            "event.v1_to_v2",
            SchemaFamily::Event,
            1,
            2,
        ))
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
    assert_eq!(
        outcome.report.diagnostics[0].severity,
        MigrationDiagnosticSeverity::Blocked
    );
    assert_eq!(outcome.report.diagnostics[0].code, "migration.missing_step");
}

#[test]
fn schema_migration_report_serializes_stable_shape() {
    let report = ruse_storage::MigrationReport {
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
        object.insert(
            "permissions".to_string(),
            json!(["file.read", "shell.exec"]),
        );
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

    assert_eq!(
        outcome.report.diagnostics[0].code,
        "config.permission_broadened"
    );
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
