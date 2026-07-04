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
    let root = std::env::temp_dir().join(format!("agent-mcp-preflight-{test_name}-{unique}"));
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
fn mcp_server_profile_preflight_diagnostics_diagnoses_workspace_requirements() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.git_status", "enabled": false}]
            }
          ]
        }"#,
    )
    .expect("manifest");
    let missing_workspace = inspect_mcp_server_export_profile(
        &manifest,
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: temp_root("missing-workspace"),
            workspace_root: None,
        },
    )
    .expect("missing workspace inspection");
    let invalid_workspace = inspect_mcp_server_export_profile(
        &manifest,
        McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: temp_root("invalid-workspace"),
            workspace_root: Some(temp_root("invalid-workspace").join("missing")),
        },
    )
    .expect("invalid workspace inspection");

    assert!(diagnostic_codes(&missing_workspace).contains(&"workspace_required".to_string()));
    assert!(diagnostic_codes(&invalid_workspace).contains(&"workspace_invalid".to_string()));
}

#[test]
fn mcp_server_profile_preflight_diagnostics_diagnoses_unknown_tools_and_missing_resources() {
    let root = temp_root("unknown-tool-missing-resource");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.shell_command"}],
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
            manifest_dir: root.clone(),
            workspace_root: Some(root),
        },
    )
    .expect("inspection");
    let unknown_tool = inspection
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "tool_unknown")
        .expect("unknown tool diagnostic");

    assert_eq!(
        unknown_tool.severity,
        McpServerProfileInspectionSeverity::Error
    );
    assert!(unknown_tool.message.contains("workspace.shell_command"));
    assert!(unknown_tool.message.contains("workspace.read_file"));
    assert!(diagnostic_codes(&inspection).contains(&"resource_missing".to_string()));
}

#[test]
fn mcp_server_profile_preflight_diagnostics_warns_for_disabled_hidden_exports() {
    let root = temp_root("disabled-hidden");
    write_file(&root, "hidden.txt", "hidden content");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.read_file", "enabled": false}],
              "resources": [
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
    let codes = diagnostic_codes(&inspection);

    assert!(codes.contains(&"tool_disabled".to_string()));
    assert!(codes.contains(&"resource_disabled".to_string()));
    assert!(inspection.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "tool_disabled"
            && diagnostic.severity == McpServerProfileInspectionSeverity::Warning
    }));
    assert!(
        !serde_json::to_string(&inspection)
            .expect("json")
            .contains("hidden content")
    );
}

fn diagnostic_codes(inspection: &agent_mcp::McpServerProfileInspection) -> Vec<String> {
    inspection
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}
