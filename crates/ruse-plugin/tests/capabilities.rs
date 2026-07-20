use ruse_plugin::{
    PluginCapabilityKind, PluginManifest, PluginManifestError, parse_plugin_manifest_json,
    permission_request_for_plugin_tool, validate_plugin_capabilities,
};
use ruse_policy::{CapabilityAction, ResourceKind};
use ruse_tool::ToolName;
use ruse_types::{ActorRef, CorrelationId};
use serde_json::{Value, json};

const TEMPLATE: &str = include_str!("../../../docs/templates/plugin-manifest.template.json");

#[test]
fn plugin_capabilities_validate_tool_and_hook_declarations() {
    let manifest = manifest_with_hook();

    let set = validate_plugin_capabilities(&manifest).expect("validate capabilities");

    assert_eq!(set.plugin_name.as_str(), "example-plugin");
    assert_eq!(set.capabilities.len(), 2);
    assert_eq!(set.capabilities[0].kind, PluginCapabilityKind::Tool);
    assert_eq!(set.capabilities[0].permissions[0].resource, "fs");
    assert_eq!(set.capabilities[0].permissions[0].action, "read");
    assert_eq!(set.capabilities[1].kind, PluginCapabilityKind::Hook);
}

#[test]
fn plugin_capabilities_reject_missing_tool_capability() {
    let mut value = template_value();
    value["capabilities"] = json!([]);
    let manifest = parse_plugin_manifest_json(&value.to_string()).expect("parse manifest");

    let err = validate_plugin_capabilities(&manifest).unwrap_err();

    assert!(matches!(err, PluginManifestError::InvalidManifest(_)));
}

#[test]
fn plugin_capabilities_reject_missing_hook_capability() {
    let mut value = template_value();
    value["hooks"] = json!([{ "name": "audit", "event": "turn_started" }]);
    let manifest = parse_plugin_manifest_json(&value.to_string()).expect("parse manifest");

    let err = validate_plugin_capabilities(&manifest).unwrap_err();

    assert!(matches!(err, PluginManifestError::InvalidManifest(_)));
}

#[test]
fn plugin_capabilities_reject_duplicate_capability() {
    let mut value = template_value();
    let capability = value["capabilities"][0].clone();
    value["capabilities"]
        .as_array_mut()
        .expect("capabilities array")
        .push(capability);
    let manifest = parse_plugin_manifest_json(&value.to_string()).expect("parse manifest");

    let err = validate_plugin_capabilities(&manifest).unwrap_err();

    assert!(matches!(err, PluginManifestError::DuplicateContribution(_)));
}

#[test]
fn plugin_capabilities_reject_empty_permission_parts() {
    let mut manifest = parse_plugin_manifest_json(TEMPLATE).expect("parse manifest");
    manifest.capabilities[0].permissions[0].action = " ".to_string();

    let err = validate_plugin_capabilities(&manifest).unwrap_err();

    assert!(matches!(err, PluginManifestError::InvalidManifest(_)));
}

#[test]
fn plugin_capabilities_tool_permission_request_maps_to_tool_scope() {
    let plugin_name = parse_plugin_manifest_json(TEMPLATE)
        .expect("parse manifest")
        .name;
    let tool_name = ToolName::new("example.read").expect("valid tool name");
    let actor = ActorRef::system();
    let correlation_id = CorrelationId::new();

    let request =
        permission_request_for_plugin_tool(&plugin_name, &tool_name, actor.clone(), correlation_id)
            .expect("permission request");

    assert_eq!(request.actor, actor);
    assert_eq!(request.action, CapabilityAction::Read);
    assert_eq!(request.scope.kind, ResourceKind::Tool);
    assert_eq!(request.scope.pattern, "example.read");
    assert_eq!(
        request.reason,
        "plugin tool execution requested: example-plugin"
    );
    assert_eq!(request.correlation_id, correlation_id);
}

fn manifest_with_hook() -> PluginManifest {
    let mut value = template_value();
    value["capabilities"]
        .as_array_mut()
        .expect("capabilities array")
        .push(json!({
            "kind": "hook",
            "name": "audit",
            "permissions": ["runtime.read"]
        }));
    value["hooks"] = json!([{ "name": "audit", "event": "turn_started" }]);
    parse_plugin_manifest_json(&value.to_string()).expect("parse manifest")
}

fn template_value() -> Value {
    serde_json::from_str(TEMPLATE).expect("parse template json")
}
