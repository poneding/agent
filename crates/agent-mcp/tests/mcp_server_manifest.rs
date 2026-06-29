use agent_mcp::{
    MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION, McpServerExportManifest,
    McpServerExportProfileConfig, McpServerResourceExportConfig, McpServerToolExportConfig,
    parse_mcp_server_export_manifest_json, select_mcp_server_export_profile,
};

#[test]
fn mcp_server_manifest_serializes_stable_shape() {
    let manifest = McpServerExportManifest {
        schema_version: MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION,
        profiles: vec![McpServerExportProfileConfig {
            name: "default".to_string(),
            tools: vec![McpServerToolExportConfig {
                name: "test.echo".to_string(),
                enabled: true,
            }],
            resources: vec![McpServerResourceExportConfig {
                uri: "agent://readme".to_string(),
                name: "README".to_string(),
                path: "README.md".to_string(),
                mime_type: Some("text/markdown".to_string()),
                enabled: true,
                sensitive: false,
            }],
        }],
    };

    let value = serde_json::to_value(&manifest).expect("manifest json");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["profiles"][0]["name"], "default");
    assert_eq!(value["profiles"][0]["tools"][0]["name"], "test.echo");
    assert_eq!(
        value["profiles"][0]["resources"][0]["uri"],
        "agent://readme"
    );
}

#[test]
fn mcp_server_manifest_parses_valid_json_and_selects_profiles() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "test.echo"}],
              "resources": [
                {
                  "uri": "agent://readme",
                  "name": "README",
                  "path": "README.md",
                  "mime_type": "text/markdown"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("parse manifest");

    let selected =
        select_mcp_server_export_profile(&manifest, None).expect("select default profile");

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(selected.name, "default");
    assert_eq!(selected.tools[0].name, "test.echo");
    assert!(selected.tools[0].enabled);
    assert_eq!(selected.resources[0].uri, "agent://readme");
    assert!(selected.resources[0].enabled);
    assert!(!selected.resources[0].sensitive);
}

#[test]
fn mcp_server_manifest_rejects_unsupported_schema_and_empty_profiles() {
    let unsupported = parse_mcp_server_export_manifest_json(
        r#"{"schema_version": 99, "profiles": [{"name": "default"}]}"#,
    )
    .expect_err("unsupported schema");
    let empty = parse_mcp_server_export_manifest_json(r#"{"schema_version": 1, "profiles": []}"#)
        .expect_err("empty manifest");

    assert!(unsupported.to_string().contains("unsupported"));
    assert!(empty.to_string().contains("at least one profile"));
}

#[test]
fn mcp_server_manifest_rejects_duplicates() {
    let duplicate_profiles = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {"name": "default"},
            {"name": "default"}
          ]
        }"#,
    )
    .expect_err("duplicate profile");
    let duplicate_tools = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "test.echo"}, {"name": "test.echo"}]
            }
          ]
        }"#,
    )
    .expect_err("duplicate tool");
    let duplicate_resources = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {"uri": "agent://readme", "name": "README", "path": "README.md"},
                {"uri": "agent://readme", "name": "README copy", "path": "README-copy.md"}
              ]
            }
          ]
        }"#,
    )
    .expect_err("duplicate resource");

    assert!(duplicate_profiles.to_string().contains("duplicate"));
    assert!(duplicate_tools.to_string().contains("duplicate"));
    assert!(duplicate_resources.to_string().contains("duplicate"));
}

#[test]
fn mcp_server_manifest_rejects_secret_like_values_without_echoing_them() {
    let error = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://readme",
                  "name": "README",
                  "path": "token/sk-secret"
                }
              ]
            }
          ]
        }"#,
    )
    .expect_err("secret-like path");
    let rendered = error.to_string();

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-secret"));
    assert!(!rendered.contains("token/sk"));
}

#[test]
fn mcp_server_manifest_selection_reports_unknown_profile_safely() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [{"name": "default"}]
        }"#,
    )
    .expect("parse");

    let error = select_mcp_server_export_profile(&manifest, Some("api_key sk-secret"))
        .expect_err("unknown profile");
    let rendered = error.to_string();

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-secret"));
}
