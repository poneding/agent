use ruse_mcp::{
    McpServerResourceTemplateExportConfig, build_mcp_server_resource_profile_from_manifest,
    inspect_mcp_server_export_profile, parse_mcp_server_export_manifest_json,
};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_types::{ActorRef, CorrelationId};
use tempfile::TempDir;

fn allow_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    )
}

#[test]
fn mcp_server_manifest_resource_templates_parse_and_serialize_stable_shape() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "description": "Read a workspace file",
                  "path_template": "{path}",
                  "mime_type": "text/plain",
                  "enabled": true,
                  "sensitive": false
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    assert_eq!(manifest.profiles[0].resource_templates.len(), 1);
    assert_eq!(
        manifest.profiles[0].resource_templates[0],
        McpServerResourceTemplateExportConfig {
            uri_template: "agent://workspace/file/{path}".to_string(),
            name: "Workspace file".to_string(),
            description: Some("Read a workspace file".to_string()),
            path_template: "{path}".to_string(),
            variable_completions: Vec::new(),
            mime_type: Some("text/plain".to_string()),
            enabled: true,
            sensitive: false,
        }
    );
    let value = serde_json::to_value(&manifest).expect("manifest json");
    assert_eq!(
        value["profiles"][0]["resource_templates"][0]["uri_template"],
        "agent://workspace/file/{path}"
    );
}

#[test]
fn mcp_server_manifest_resource_templates_builds_visible_file_template_profile() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "hello template").expect("fixture");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "description": "Read a workspace file",
                  "path_template": "{path}",
                  "mime_type": "text/plain"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_resource_profile_from_manifest(&manifest, None, temp.path())
        .expect("runtime");
    let templates = runtime.resource_exports().list_resource_templates();

    assert_eq!(runtime.profile().resource_template_uri_templates.len(), 1);
    assert_eq!(templates[0].uri_template, "agent://workspace/file/{path}");
    let read = runtime
        .resource_exports()
        .read_resource(
            &allow_policy("agent://workspace/file/{path}"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/README.md",
        )
        .expect("read");
    assert_eq!(read.contents, "hello template");
}

#[test]
fn mcp_server_manifest_resource_templates_hide_disabled_templates_but_inspect_them() {
    let temp = TempDir::new().expect("tempdir");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "path_template": "{path}",
                  "enabled": false
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_resource_profile_from_manifest(&manifest, None, temp.path())
        .expect("runtime");
    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        ruse_mcp::McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: temp.path().to_path_buf(),
            workspace_root: None,
        },
    )
    .expect("inspection");

    assert!(runtime.profile().resource_template_uri_templates.is_empty());
    assert!(
        runtime
            .resource_exports()
            .list_resource_templates()
            .is_empty()
    );
    assert_eq!(inspection.resource_templates.len(), 1);
    assert!(!inspection.resource_templates[0].enabled);
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "resource_template_disabled")
    );
}

#[test]
fn mcp_server_manifest_resource_templates_reject_duplicate_templates() {
    let error = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "path_template": "{path}"
                },
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file copy",
                  "path_template": "{path}"
                }
              ]
            }
          ]
        }"#,
    )
    .expect_err("duplicate");

    assert!(error.to_string().contains("duplicate"));
}

#[test]
fn mcp_server_manifest_resource_templates_inspection_redacts_metadata_without_reading_contents() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "api_key sk-secret").expect("fixture");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "description": "Read workspace files",
                  "path_template": "{path}",
                  "sensitive": true
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        ruse_mcp::McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: temp.path().to_path_buf(),
            workspace_root: None,
        },
    )
    .expect("inspection");
    let rendered = serde_json::to_string(&inspection).expect("inspection json");

    assert_eq!(inspection.resource_templates[0].path_template, "{path}");
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("sk-secret"));
}
