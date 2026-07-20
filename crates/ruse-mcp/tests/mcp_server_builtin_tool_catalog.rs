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
    std::env::temp_dir().join(format!("ruse-mcp-{name}-{nanos}"))
}

#[test]
fn mcp_server_builtin_tool_catalog_registers_curated_read_only_tools() {
    let root = unique_workspace("catalog");
    fs::create_dir_all(&root).expect("workspace dir");

    let catalog = build_mcp_server_builtin_tool_export_catalog(
        McpServerBuiltinToolExportCatalogConfig::new(&root),
    )
    .expect("builtin catalog");

    assert_eq!(
        catalog.tool_names(),
        [
            "workspace.git_diff",
            "workspace.git_status",
            "workspace.read_file",
            "workspace.search_text"
        ]
    );
}

#[test]
fn mcp_server_builtin_tool_catalog_excludes_side_effecting_tools() {
    let root = unique_workspace("excluded");
    fs::create_dir_all(&root).expect("workspace dir");

    let catalog = build_mcp_server_builtin_tool_export_catalog(
        McpServerBuiltinToolExportCatalogConfig::new(&root),
    )
    .expect("builtin catalog");
    let names = catalog.tool_names();

    assert!(!names.contains(&"workspace.shell_command".to_string()));
    assert!(!names.contains(&"workspace.edit_file".to_string()));
}

#[test]
fn mcp_server_builtin_tool_catalog_rejects_invalid_workspace_root() {
    let missing = unique_workspace("missing");

    let error = match build_mcp_server_builtin_tool_export_catalog(
        McpServerBuiltinToolExportCatalogConfig::new(&missing),
    ) {
        Ok(_) => panic!("missing workspace rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("workspace root"));
    assert!(!error.to_string().contains("api_key"));
}

#[test]
fn mcp_server_builtin_tool_catalog_binds_manifest_tool() {
    let root = unique_workspace("bind");
    fs::create_dir_all(&root).expect("workspace dir");
    fs::write(root.join("README.md"), "hello mcp\n").expect("fixture file");
    let catalog = build_mcp_server_builtin_tool_export_catalog(
        McpServerBuiltinToolExportCatalogConfig::new(&root),
    )
    .expect("builtin catalog");
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

    let runtime =
        build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog).expect("profile");

    assert_eq!(runtime.profile().tool_names, ["workspace.read_file"]);
    assert_eq!(
        runtime.tool_exports().list_tools()[0].name,
        "workspace.read_file"
    );
}
