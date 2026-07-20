use ruse_plugin::{
    PluginIsolationMode, PluginManifestError, check_plugin_compatibility,
    parse_plugin_manifest_json, validate_plugin_manifest,
};
use serde_json::{Value, json};

const TEMPLATE: &str = include_str!("../../../docs/templates/plugin-manifest.template.json");

#[test]
fn manifest_parses_template_and_validates() {
    let manifest = parse_plugin_manifest_json(TEMPLATE).expect("parse template manifest");

    validate_plugin_manifest(&manifest).expect("template manifest validates");

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.name.as_str(), "example-plugin");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.runtime_compatibility.api_version, 1);
    assert_eq!(manifest.isolation.mode, PluginIsolationMode::Process);
    assert_eq!(
        manifest.isolation.entrypoint.as_deref(),
        Some("bin/example-plugin")
    );
    assert_eq!(manifest.capabilities.len(), 1);
    assert_eq!(manifest.tools.len(), 1);
    assert!(manifest.hooks.is_empty());
}

#[test]
fn manifest_rejects_unsupported_schema_version() {
    let mut value = template_value();
    value["schema_version"] = json!(2);

    let err = parse_plugin_manifest_json(&value.to_string()).unwrap_err();

    assert!(matches!(
        err,
        PluginManifestError::UnsupportedSchema {
            found: 2,
            supported: 1
        }
    ));
}

#[test]
fn manifest_rejects_unsupported_api_version() {
    let mut manifest = parse_plugin_manifest_json(TEMPLATE).expect("parse template manifest");
    manifest.runtime_compatibility.api_version = 99;

    let err = check_plugin_compatibility(&manifest.runtime_compatibility).unwrap_err();

    assert!(matches!(err, PluginManifestError::IncompatibleRuntime(_)));
}

#[test]
fn manifest_rejects_invalid_plugin_name() {
    let mut value = template_value();
    value["name"] = json!("Bad Plugin Name");

    let err = parse_plugin_manifest_json(&value.to_string()).unwrap_err();

    assert!(matches!(err, PluginManifestError::InvalidName(_)));
}

#[test]
fn manifest_rejects_missing_process_entrypoint() {
    let mut value = template_value();
    value["isolation"]["entrypoint"] = Value::Null;

    let manifest = parse_plugin_manifest_json(&value.to_string()).expect("parse manifest");
    let err = validate_plugin_manifest(&manifest).unwrap_err();

    assert!(matches!(err, PluginManifestError::InvalidIsolation(_)));
}

#[test]
fn manifest_rejects_builtin_entrypoint() {
    let mut value = template_value();
    value["isolation"]["mode"] = json!("builtin");

    let manifest = parse_plugin_manifest_json(&value.to_string()).expect("parse manifest");
    let err = validate_plugin_manifest(&manifest).unwrap_err();

    assert!(matches!(err, PluginManifestError::InvalidIsolation(_)));
}

#[test]
fn manifest_rejects_duplicate_tools_and_hooks() {
    let mut duplicate_tools = template_value();
    let tool = duplicate_tools["tools"][0].clone();
    duplicate_tools["tools"]
        .as_array_mut()
        .expect("tools array")
        .push(tool);
    let manifest =
        parse_plugin_manifest_json(&duplicate_tools.to_string()).expect("parse duplicate tools");
    let err = validate_plugin_manifest(&manifest).unwrap_err();
    assert!(matches!(err, PluginManifestError::DuplicateContribution(_)));

    let mut duplicate_hooks = template_value();
    duplicate_hooks["hooks"] = json!([
        { "name": "audit", "event": "turn_started" },
        { "name": "audit", "event": "turn_completed" }
    ]);
    let manifest =
        parse_plugin_manifest_json(&duplicate_hooks.to_string()).expect("parse duplicate hooks");
    let err = validate_plugin_manifest(&manifest).unwrap_err();
    assert!(matches!(err, PluginManifestError::DuplicateContribution(_)));
}

#[test]
fn manifest_serializes_stable_shape() {
    let manifest = parse_plugin_manifest_json(TEMPLATE).expect("parse template manifest");

    let value = serde_json::to_value(&manifest).expect("serialize manifest");

    assert_eq!(value["schema_version"], json!(1));
    assert_eq!(value["name"], json!("example-plugin"));
    assert_eq!(
        value["runtime_compatibility"]["min_agent_version"],
        json!("0.1.0")
    );
    assert_eq!(value["runtime_compatibility"]["api_version"], json!(1));
    assert_eq!(value["isolation"]["mode"], json!("process"));
    assert_eq!(
        value["isolation"]["entrypoint"],
        json!("bin/example-plugin")
    );
    assert_eq!(value["capabilities"][0]["kind"], json!("tool"));
    assert_eq!(value["capabilities"][0]["name"], json!("example.read"));
    assert_eq!(value["capabilities"][0]["permissions"][0], json!("fs.read"));
    assert_eq!(value["tools"][0]["name"], json!("example.read"));
}

fn template_value() -> Value {
    serde_json::from_str(TEMPLATE).expect("parse template json")
}
