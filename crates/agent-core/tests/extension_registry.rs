use agent_core::{
    EXTENSION_HEALTH_SMOKE_SCHEMA_VERSION, ExtensionCapabilityLease, ExtensionCapabilitySummary,
    ExtensionCompatibility, ExtensionHealth, ExtensionHealthSmokeDiagnosticSeverity,
    ExtensionHealthStatus, ExtensionId, ExtensionKind, ExtensionPermissionSummary, ExtensionRecord,
    ExtensionRegistry, ExtensionRegistryError, ExtensionSource, ExtensionSourceKind,
    MAX_EXTENSION_HEALTH_SMOKE_DIAGNOSTICS, MAX_EXTENSION_HEALTH_SMOKE_MESSAGE_CHARS,
};
use agent_policy::{CapabilityAction, ResourceKind};
use agent_types::Timestamp;
use serde_json::json;

fn ts() -> Timestamp {
    serde_json::from_value(json!("2026-06-27T10:00:00Z")).expect("valid timestamp")
}

fn permission(resource: &str) -> ExtensionPermissionSummary {
    ExtensionPermissionSummary::new(ResourceKind::Tool, CapabilityAction::Execute, resource)
        .expect("valid permission")
}

fn capability(name: &str) -> ExtensionCapabilitySummary {
    ExtensionCapabilitySummary::new("tool", name, vec![permission(name)]).expect("valid capability")
}

fn lease(id: &str) -> ExtensionCapabilityLease {
    ExtensionCapabilityLease::new(
        id,
        "read",
        ResourceKind::Tool,
        CapabilityAction::Execute,
        "example.read",
        Some(ts()),
    )
    .expect("valid lease")
}

fn lease_for(id: &str, capability_name: &str) -> ExtensionCapabilityLease {
    ExtensionCapabilityLease::new(
        id,
        capability_name,
        ResourceKind::Tool,
        CapabilityAction::Execute,
        "example.read",
        Some(ts()),
    )
    .expect("valid lease")
}

fn source(kind: ExtensionSourceKind, reference: &str) -> ExtensionSource {
    ExtensionSource::new(kind, reference, reference, Some("sha256:fixture")).expect("valid source")
}

fn record(id: &str, kind: ExtensionKind, enabled: bool) -> ExtensionRecord {
    ExtensionRecord::new(
        ExtensionId::new(id).expect("valid id"),
        kind,
        source(ExtensionSourceKind::LocalPath, &format!("/tmp/{id}")),
        enabled,
        Some("0.1.0".to_string()),
        ExtensionCompatibility::supported(1),
        vec![capability("example.read")],
        vec![lease("lease-1")],
        ExtensionHealth::healthy("ready"),
    )
    .expect("valid record")
}

#[test]
fn extension_registry_accepts_all_extension_kinds_and_serializes_snapshot() {
    let registry = ExtensionRegistry::new(vec![
        record("plugin.alpha", ExtensionKind::Plugin, true),
        record("skill.root", ExtensionKind::SkillRoot, true),
        record("mcp.docs", ExtensionKind::McpServer, false),
    ])
    .expect("valid registry");

    let snapshot = registry.snapshot();
    let ids = snapshot
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["mcp.docs", "plugin.alpha", "skill.root"]);
    assert_eq!(
        serde_json::to_value(&snapshot.records[0]).expect("snapshot json"),
        json!({
            "id": "mcp.docs",
            "kind": "mcp_server",
            "source": {
                "kind": "local_path",
                "reference": "/tmp/mcp.docs",
                "display_label": "/tmp/mcp.docs",
                "content_hash": "sha256:fixture"
            },
            "enabled": false,
            "version": "0.1.0",
            "compatibility": {
                "status": "supported",
                "api_version": 1
            },
            "declared_capabilities": [{
                "kind": "tool",
                "name": "example.read",
                "permissions": [{
                    "resource_kind": "tool",
                    "action": "execute",
                    "resource": "example.read"
                }]
            }],
            "granted_leases": [{
                "lease_id": "lease-1",
                "capability_name": "read",
                "resource_kind": "tool",
                "action": "execute",
                "resource": "example.read",
                "expires_at": "2026-06-27T10:00:00Z"
            }],
            "health": {
                "status": "healthy",
                "message": "ready"
            }
        })
    );
}

#[test]
fn extension_registry_rejects_duplicate_ids() {
    let error = ExtensionRegistry::new(vec![
        record("plugin.alpha", ExtensionKind::Plugin, true),
        record("plugin.alpha", ExtensionKind::Plugin, false),
    ])
    .expect_err("duplicate id rejected");

    assert!(matches!(
        error,
        ExtensionRegistryError::DuplicateId { id } if id.as_str() == "plugin.alpha"
    ));
}

#[test]
fn extension_registry_upsert_replaces_existing_record_in_sorted_order() {
    let mut registry =
        ExtensionRegistry::new(vec![record("plugin.alpha", ExtensionKind::Plugin, true)])
            .expect("valid registry");
    let replacement = record("plugin.alpha", ExtensionKind::Plugin, false);

    registry.upsert(replacement).expect("upsert succeeds");

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.records.len(), 1);
    assert!(!snapshot.records[0].enabled);
}

#[test]
fn extension_registry_disable_revokes_active_leases() {
    let mut registry =
        ExtensionRegistry::new(vec![record("plugin.alpha", ExtensionKind::Plugin, true)])
            .expect("valid registry");

    let disabled = registry
        .disable(&ExtensionId::new("plugin.alpha").unwrap())
        .expect("disable succeeds");

    assert!(!disabled.enabled);
    assert!(disabled.granted_leases.is_empty());
    assert_eq!(disabled.health.status, ExtensionHealthStatus::Disabled);
    assert!(registry.snapshot().records[0].granted_leases.is_empty());
}

#[test]
fn extension_registry_remove_returns_removed_record() {
    let mut registry =
        ExtensionRegistry::new(vec![record("plugin.alpha", ExtensionKind::Plugin, true)])
            .expect("valid registry");

    let removed = registry
        .remove(&ExtensionId::new("plugin.alpha").unwrap())
        .expect("remove succeeds");

    assert_eq!(removed.id.as_str(), "plugin.alpha");
    assert!(registry.snapshot().records.is_empty());
}

#[test]
fn unsupported_compatibility_disables_record_and_revokes_leases() {
    let unsupported = ExtensionRecord::new(
        ExtensionId::new("plugin.alpha").expect("valid id"),
        ExtensionKind::Plugin,
        source(ExtensionSourceKind::LocalPath, "/tmp/plugin.alpha"),
        true,
        Some("9.0.0".to_string()),
        ExtensionCompatibility::unsupported(99, 1, "runtime api is too new"),
        vec![capability("example.read")],
        vec![lease("lease-1")],
        ExtensionHealth::healthy("ready"),
    )
    .expect("record is normalized");

    assert!(!unsupported.enabled);
    assert!(unsupported.granted_leases.is_empty());
    assert_eq!(unsupported.health.status, ExtensionHealthStatus::Unhealthy);
    assert!(
        unsupported
            .health
            .message
            .contains("runtime api is too new")
    );
}

#[test]
fn extension_registry_redacts_secret_like_source_health_and_capabilities() {
    let record = ExtensionRecord::new(
        ExtensionId::new("plugin.redacted").expect("valid id"),
        ExtensionKind::Plugin,
        source(
            ExtensionSourceKind::LocalPath,
            "/tmp/plugin?token=sk-secret&password=pw",
        ),
        true,
        Some("0.1.0".to_string()),
        ExtensionCompatibility::supported(1),
        vec![
            ExtensionCapabilitySummary::new(
                "tool",
                "api_key.reader",
                vec![permission("token sk-secret password")],
            )
            .expect("valid redacted capability"),
        ],
        Vec::new(),
        ExtensionHealth::degraded("api_key sk-secret token password should not leak"),
    )
    .expect("valid record");

    let serialized = serde_json::to_string(&record).expect("record json");

    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
}

#[test]
fn extension_registry_rejects_invalid_capability_summary() {
    assert!(ExtensionId::new("Plugin Alpha").is_err());
    assert!(ExtensionCapabilitySummary::new("tool", " ", vec![permission("read")]).is_err());
    assert!(
        ExtensionPermissionSummary::new(ResourceKind::Tool, CapabilityAction::Execute, " ")
            .is_err()
    );
}

#[test]
fn extension_health_smoke_report_summarizes_all_kinds_with_stable_actions() {
    let plugin = ExtensionRecord::new(
        ExtensionId::new("plugin.alpha").expect("valid id"),
        ExtensionKind::Plugin,
        source(ExtensionSourceKind::LocalPath, "/tmp/plugin.alpha"),
        true,
        Some("0.1.0".to_string()),
        ExtensionCompatibility::supported(1),
        vec![capability("example.read")],
        vec![lease_for("lease-1", "example.read")],
        ExtensionHealth::healthy("ready"),
    )
    .expect("valid plugin");
    let skill_root = ExtensionRecord::new(
        ExtensionId::new("skill.root").expect("valid id"),
        ExtensionKind::SkillRoot,
        source(ExtensionSourceKind::LocalPath, "/tmp/skill.root"),
        true,
        Some("0.1.0".to_string()),
        ExtensionCompatibility::supported(1),
        vec![capability("example.read")],
        Vec::new(),
        ExtensionHealth::degraded("review token sk-secret before enablement"),
    )
    .expect("valid skill root");
    let mcp_server = ExtensionRecord::new(
        ExtensionId::new("mcp.docs").expect("valid id"),
        ExtensionKind::McpServer,
        source(ExtensionSourceKind::Config, "mcp.docs"),
        true,
        Some("9.0.0".to_string()),
        ExtensionCompatibility::unsupported(9, 1, "api_key sk-secret is too new"),
        vec![capability("example.read")],
        vec![lease_for("lease-2", "example.read")],
        ExtensionHealth::healthy("ready"),
    )
    .expect("valid mcp server");
    let disabled = record("plugin.disabled", ExtensionKind::Plugin, true).disabled();
    let registry = ExtensionRegistry::new(vec![skill_root, plugin, mcp_server, disabled])
        .expect("valid registry");

    let report = registry.health_smoke_report();
    let serialized = serde_json::to_string(&report).expect("report json");

    assert_eq!(report.schema_version, EXTENSION_HEALTH_SMOKE_SCHEMA_VERSION);
    assert_eq!(report.total_extensions, 4);
    assert_eq!(
        serde_json::to_value(&report.by_kind).expect("kind counts"),
        json!([
            { "kind": "plugin", "count": 2 },
            { "kind": "skill_root", "count": 1 },
            { "kind": "mcp_server", "count": 1 }
        ])
    );
    assert_eq!(
        serde_json::to_value(&report.by_status).expect("status counts"),
        json!([
            { "status": "healthy", "count": 1 },
            { "status": "degraded", "count": 1 },
            { "status": "unhealthy", "count": 1 },
            { "status": "disabled", "count": 1 },
            { "status": "unknown", "count": 0 }
        ])
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "extension.unsupported_compatibility")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "extension.capability_review_required")
    );
    assert_eq!(
        report.next_actions,
        vec![
            "extension.enable_when_ready".to_string(),
            "extension.inspect_unhealthy".to_string(),
            "extension.review_capabilities".to_string()
        ]
    );
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("token"));
}

#[test]
fn extension_health_smoke_report_handles_empty_registry() {
    let registry = ExtensionRegistry::default();

    let report = registry.health_smoke_report();

    assert_eq!(report.total_extensions, 0);
    assert_eq!(report.next_actions, vec!["extension.add_first"]);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, "extension.none_configured");
    assert_eq!(
        report.diagnostics[0].severity,
        ExtensionHealthSmokeDiagnosticSeverity::Info
    );
}

#[test]
fn extension_health_smoke_report_bounds_diagnostics_and_messages() {
    let records = (0..24)
        .map(|index| {
            ExtensionRecord::new(
                ExtensionId::new(format!("plugin.noisy-{index:02}")).expect("valid id"),
                ExtensionKind::Plugin,
                source(
                    ExtensionSourceKind::LocalPath,
                    &format!("/tmp/plugin.noisy-{index:02}"),
                ),
                true,
                Some("0.1.0".to_string()),
                ExtensionCompatibility::supported(1),
                vec![capability("example.read")],
                Vec::new(),
                ExtensionHealth::unhealthy(format!(
                    "{} sk-secret token password",
                    "very long diagnostic message ".repeat(20)
                )),
            )
            .expect("valid noisy record")
        })
        .collect::<Vec<_>>();
    let registry = ExtensionRegistry::new(records).expect("valid registry");

    let report = registry.health_smoke_report();
    let serialized = serde_json::to_string(&report).expect("report json");

    assert_eq!(
        report.diagnostics.len(),
        MAX_EXTENSION_HEALTH_SMOKE_DIAGNOSTICS
    );
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.code
        == "extension.diagnostics_truncated"
        && diagnostic.truncated));
    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message.chars().count()
                <= MAX_EXTENSION_HEALTH_SMOKE_MESSAGE_CHARS + 3)
    );
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
}
