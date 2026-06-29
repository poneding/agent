use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_mcp::{
    McpServerProfileInspectionOptions, McpServerProfileInspectionSeverity,
    inspect_mcp_server_export_profile, parse_mcp_server_export_manifest_json,
};

fn temp_root(test_name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("agent-mcp-inspection-{test_name}-{unique}"));
    fs::create_dir_all(&root).expect("temp root");
    root
}

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent");
    }
    fs::write(path, contents).expect("write fixture");
}

#[test]
fn mcp_server_profile_inspection_reports_resource_metadata_without_contents() {
    let root = temp_root("resource-metadata");
    write_file(
        &root,
        "docs/README.md",
        "classified contents should not leak",
    );
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://readme",
                  "name": "README",
                  "path": "docs/README.md",
                  "mime_type": "text/markdown",
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
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: root,
            workspace_root: None,
        },
    )
    .expect("inspection");
    let rendered = serde_json::to_string(&inspection).expect("json");

    assert_eq!(inspection.profile_name, "default");
    assert!(inspection.tools.is_empty());
    assert!(inspection.diagnostics.is_empty());
    assert_eq!(inspection.resources.len(), 1);
    assert_eq!(inspection.resources[0].uri, "agent://readme");
    assert_eq!(inspection.resources[0].name, "README");
    assert_eq!(inspection.resources[0].path, "docs/README.md");
    assert_eq!(
        inspection.resources[0].mime_type,
        Some("text/markdown".to_string())
    );
    assert!(inspection.resources[0].enabled);
    assert!(inspection.resources[0].sensitive);
    assert!(!rendered.contains("classified contents"));
}

#[test]
fn mcp_server_profile_inspection_diagnoses_missing_workspace_for_enabled_tools() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.read_file"}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: temp_root("missing-workspace"),
            workspace_root: None,
        },
    )
    .expect("inspection");

    assert_eq!(inspection.tools.len(), 1);
    assert_eq!(inspection.tools[0].name, "workspace.read_file");
    assert!(inspection.tools[0].enabled);
    assert_eq!(inspection.diagnostics.len(), 1);
    assert_eq!(
        inspection.diagnostics[0].severity,
        McpServerProfileInspectionSeverity::Error
    );
    assert_eq!(inspection.diagnostics[0].code, "workspace_required");
    assert!(inspection.diagnostics[0].message.contains("--workspace"));
}

#[test]
fn mcp_server_profile_inspection_includes_disabled_entries() {
    let root = temp_root("disabled-entries");
    write_file(&root, "visible.txt", "visible");
    write_file(&root, "hidden.txt", "hidden");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [
                {"name": "workspace.read_file"},
                {"name": "workspace.git_status", "enabled": false}
              ],
              "resources": [
                {"uri": "agent://visible", "name": "Visible", "path": "visible.txt"},
                {"uri": "agent://hidden", "name": "Hidden", "path": "hidden.txt", "enabled": false}
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: root.clone(),
            workspace_root: Some(root),
        },
    )
    .expect("inspection");
    let enabled_tools = inspection.tools.iter().filter(|tool| tool.enabled).count();
    let enabled_resources = inspection
        .resources
        .iter()
        .filter(|resource| resource.enabled)
        .count();

    assert_eq!(inspection.tools.len(), 2);
    assert_eq!(enabled_tools, 1);
    assert_eq!(inspection.resources.len(), 2);
    assert_eq!(enabled_resources, 1);
    assert_eq!(inspection.tools[1].name, "workspace.git_status");
    assert!(!inspection.tools[1].enabled);
    assert_eq!(inspection.resources[1].uri, "agent://hidden");
    assert!(!inspection.resources[1].enabled);
}

#[test]
fn mcp_server_profile_inspection_serializes_stable_snake_case_shape() {
    let root = temp_root("serde-shape");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.search_text"}],
              "resources": [
                {"uri": "agent://missing", "name": "Missing", "path": "missing.txt"}
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: root,
            workspace_root: None,
        },
    )
    .expect("inspection");
    let value = serde_json::to_value(&inspection).expect("json");

    assert_eq!(value["profile_name"], "default");
    assert_eq!(value["tools"][0]["name"], "workspace.search_text");
    assert_eq!(value["tools"][0]["enabled"], true);
    assert_eq!(value["resources"][0]["uri"], "agent://missing");
    assert_eq!(value["resources"][0]["path"], "missing.txt");
    assert_eq!(value["resources"][0]["enabled"], true);
    assert_eq!(value["resources"][0]["sensitive"], false);
    let workspace_required = value["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .find(|diagnostic| diagnostic["code"] == "workspace_required")
        .expect("workspace diagnostic");
    assert_eq!(workspace_required["severity"], "error");
}

#[test]
fn mcp_server_profile_inspection_redacts_secret_like_diagnostics() {
    let root = temp_root("redacted-diagnostics");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.read_file"}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: root.clone(),
            workspace_root: Some(root.join("token/sk-secret-workspace")),
        },
    )
    .expect("inspection");
    let rendered = serde_json::to_string(&inspection).expect("json");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-secret"));
    assert!(!rendered.contains("token/"));
}
