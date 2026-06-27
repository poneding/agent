use agent_core::{
    ExtensionCapabilityLease, ExtensionCapabilitySummary, ExtensionCompatibility, ExtensionHealth,
    ExtensionHealthStatus, ExtensionId, ExtensionKind, ExtensionPermissionSummary, ExtensionRecord,
    ExtensionRegistry, ExtensionRegistryError, ExtensionSource, ExtensionSourceKind,
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
