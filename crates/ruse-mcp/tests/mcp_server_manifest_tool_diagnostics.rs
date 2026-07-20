use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ruse_mcp::{
    McpServerBuiltinToolExportCatalogConfig, build_mcp_server_builtin_tool_export_catalog,
    build_mcp_server_tool_profile_from_manifest, parse_mcp_server_export_manifest_json,
};

fn unique_workspace(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ruse-mcp-diagnostics-{name}-{nanos}"))
}

fn builtin_catalog() -> ruse_mcp::McpServerToolExportCatalog {
    let root = unique_workspace("catalog");
    fs::create_dir_all(&root).expect("workspace dir");
    build_mcp_server_builtin_tool_export_catalog(McpServerBuiltinToolExportCatalogConfig::new(
        &root,
    ))
    .expect("builtin catalog")
}

#[test]
fn mcp_server_manifest_tool_diagnostics_reject_duplicate_tool_declarations() {
    let error = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [
                {"name": "workspace.read_file"},
                {"name": "workspace.read_file"}
              ]
            }
          ]
        }"#,
    )
    .expect_err("duplicate tool rejected");

    assert!(
        error
            .to_string()
            .contains("duplicate MCP server export tool")
    );
    assert!(error.to_string().contains("workspace.read_file"));
}

#[test]
fn mcp_server_manifest_tool_diagnostics_list_known_names_for_unknown_tool() {
    let catalog = builtin_catalog();
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.shell_command"}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let error = match build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog) {
        Ok(_) => panic!("unknown tool rejected"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(message.contains("MCP server export catalog tool not found"));
    assert!(message.contains("workspace.shell_command"));
    assert!(message.contains("known tools:"));
    assert!(message.contains("workspace.read_file"));
    assert!(!message.contains("api_key"));
}

#[test]
fn mcp_server_manifest_tool_diagnostics_preserve_disabled_tool_hiding() {
    let catalog = builtin_catalog();
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.read_file", "enabled": false}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime =
        build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog).expect("profile");

    assert!(runtime.profile().tool_names.is_empty());
    assert!(runtime.tool_exports().list_tools().is_empty());
    assert_eq!(runtime.tools().definitions().len(), 1);
}
