use agent_mcp::{McpResourceExportRegistry, McpResourceReadResult};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_types::{ActorRef, CorrelationId};

fn allow_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    )
}

fn deny_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Deny {
            reason: "fixture deny".to_string(),
        },
    )
}

#[test]
fn mcp_server_resource_export_serializes_stable_descriptor() {
    let mut registry = McpResourceExportRegistry::default();

    registry
        .export_resource(
            "agent://workspace/readme",
            "README",
            Some("text/markdown"),
            "hello",
        )
        .expect("export resource");

    let listed = registry.list_resources();
    let value = serde_json::to_value(&listed[0]).expect("resource json");

    assert_eq!(value["uri"], "agent://workspace/readme");
    assert_eq!(value["name"], "README");
    assert_eq!(value["mime_type"], "text/markdown");
    assert_eq!(value["enabled"], true);
}

#[test]
fn mcp_server_resource_export_rejects_invalid_uris_and_secret_content_by_default() {
    let mut registry = McpResourceExportRegistry::default();

    assert!(
        registry
            .export_resource("not-a-uri", "Bad", None::<String>, "hello")
            .is_err()
    );
    assert!(
        registry
            .export_resource(
                "agent://secret",
                "Secret",
                None::<String>,
                "api_key sk-secret"
            )
            .is_err()
    );
    assert!(
        registry
            .export_sensitive_resource(
                "agent://private",
                "Private",
                None::<String>,
                "api_key sk-secret"
            )
            .is_ok()
    );
}

#[test]
fn mcp_server_resource_export_lists_enabled_resources_in_order() {
    let mut registry = McpResourceExportRegistry::default();

    registry
        .export_resource("agent://zeta", "Zeta", None::<String>, "z")
        .expect("zeta");
    registry
        .export_disabled_resource("agent://alpha", "Alpha", None::<String>, "a")
        .expect("alpha");
    registry
        .export_resource("agent://beta", "Beta", None::<String>, "b")
        .expect("beta");

    let listed = registry.list_resources();

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].uri, "agent://beta");
    assert_eq!(listed[1].uri, "agent://zeta");
}

#[test]
fn mcp_server_resource_export_rejects_duplicates() {
    let mut registry = McpResourceExportRegistry::default();

    registry
        .export_resource("agent://readme", "README", None::<String>, "hello")
        .expect("first export");
    let error = registry
        .export_resource("agent://readme", "README copy", None::<String>, "hello")
        .expect_err("duplicate export fails");

    assert!(error.to_string().contains("duplicate MCP resource export"));
}

#[test]
fn mcp_server_resource_export_denied_read_returns_no_content() {
    let mut registry = McpResourceExportRegistry::default();

    registry
        .export_resource("agent://readme", "README", None::<String>, "hello")
        .expect("export");
    let error = registry
        .read_resource(
            &deny_policy("agent://readme"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://readme",
        )
        .expect_err("denied read");

    assert!(
        error
            .to_string()
            .contains("policy denied MCP resource export")
    );
    assert!(!error.to_string().contains("hello"));
}

#[test]
fn mcp_server_resource_export_allowed_read_bounds_content() {
    let mut registry = McpResourceExportRegistry::default().with_max_content_bytes(8);

    registry
        .export_resource(
            "agent://readme",
            "README",
            Some("text/plain"),
            "hello from resource",
        )
        .expect("export");
    let output = registry
        .read_resource(
            &allow_policy("agent://readme"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://readme",
        )
        .expect("read");

    assert_eq!(
        output,
        McpResourceReadResult {
            uri: "agent://readme".to_string(),
            contents: "hello fr".to_string(),
            mime_type: Some("text/plain".to_string()),
            truncated: true,
        }
    );
}
