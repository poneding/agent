use agent_mcp::{
    MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION, McpServerExportManifest,
    McpServerExportProfileConfig, McpServerProfileInspectionOptions,
    McpServerProfileInspectionSeverity, McpServerPromptExportConfig, McpServerPromptMessageConfig,
    build_mcp_server_profile_from_manifest, inspect_mcp_server_export_profile,
    parse_mcp_server_export_manifest_json,
};

#[test]
fn mcp_server_manifest_prompts_serialize_and_parse_stable_shape() {
    let manifest = McpServerExportManifest {
        schema_version: MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION,
        profiles: vec![McpServerExportProfileConfig {
            name: "default".to_string(),
            tools: Vec::new(),
            resources: Vec::new(),
            resource_templates: Vec::new(),
            prompts: vec![McpServerPromptExportConfig {
                name: "review".to_string(),
                description: Some("Review current changes".to_string()),
                arguments: Vec::new(),
                argument_completions: Vec::new(),
                messages: vec![McpServerPromptMessageConfig {
                    role: "user".to_string(),
                    text: "Review the workspace diff".to_string(),
                }],
                enabled: true,
            }],
        }],
    };

    let value = serde_json::to_value(&manifest).expect("manifest json");
    let parsed = parse_mcp_server_export_manifest_json(&value.to_string()).expect("parse");

    assert_eq!(value["profiles"][0]["prompts"][0]["name"], "review");
    assert_eq!(
        value["profiles"][0]["prompts"][0]["messages"][0]["text"],
        "Review the workspace diff"
    );
    assert_eq!(parsed.profiles[0].prompts[0].name, "review");
    assert!(parsed.profiles[0].prompts[0].enabled);
}

#[test]
fn mcp_server_manifest_prompts_build_runtime_and_hide_disabled_entries() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "description": "Review current changes",
                  "messages": [{"role": "user", "text": "Review the workspace diff"}]
                },
                {
                  "name": "hidden",
                  "description": "Hidden prompt",
                  "enabled": false,
                  "messages": [{"role": "user", "text": "Do not list me"}]
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_profile_from_manifest(
        &manifest,
        None,
        std::env::temp_dir(),
        &agent_mcp::McpServerToolExportCatalog::default(),
    )
    .expect("runtime");
    let listed = runtime.prompt_exports().list_prompts();
    let got = runtime
        .prompt_exports()
        .get_prompt("review")
        .expect("get prompt");
    let hidden = runtime
        .prompt_exports()
        .get_prompt("hidden")
        .expect_err("disabled hidden");

    assert_eq!(runtime.profile().prompt_names, ["review"]);
    assert_eq!(listed.prompts.len(), 1);
    assert_eq!(listed.prompts[0].name, "review");
    assert_eq!(got.messages[0].content.text, "Review the workspace diff");
    assert!(hidden.to_string().contains("MCP prompt export not found"));
}

#[test]
fn mcp_server_manifest_prompts_reject_duplicates_and_invalid_messages() {
    let duplicate = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {"name": "review", "messages": [{"role": "user", "text": "one"}]},
                {"name": "review", "messages": [{"role": "user", "text": "two"}]}
              ]
            }
          ]
        }"#,
    )
    .expect_err("duplicate prompt");
    let empty_messages = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {"name": "default", "prompts": [{"name": "review", "messages": []}]}
          ]
        }"#,
    )
    .expect_err("empty prompt messages");

    assert!(duplicate.to_string().contains("duplicate"));
    assert!(
        empty_messages
            .to_string()
            .contains("requires at least one message")
    );
}

#[test]
fn mcp_server_manifest_prompts_inspection_summarizes_without_contents() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "description": "Review current changes",
                  "messages": [
                    {"role": "system", "text": "classified system content"},
                    {"role": "user", "text": "classified user content"}
                  ]
                },
                {
                  "name": "hidden",
                  "enabled": false,
                  "messages": [{"role": "user", "text": "hidden content"}]
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
            manifest_dir: std::env::temp_dir(),
            workspace_root: None,
        },
    )
    .expect("inspection");
    let rendered = serde_json::to_string(&inspection).expect("json");

    assert_eq!(inspection.prompts.len(), 2);
    assert_eq!(inspection.prompts[0].name, "review");
    assert_eq!(
        inspection.prompts[0].description,
        Some("Review current changes".to_string())
    );
    assert!(inspection.prompts[0].enabled);
    assert_eq!(inspection.prompts[0].message_count, 2);
    assert_eq!(inspection.prompts[1].name, "hidden");
    assert!(!inspection.prompts[1].enabled);
    assert!(inspection.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == McpServerProfileInspectionSeverity::Warning
            && diagnostic.code == "prompt_disabled"
    }));
    assert!(!rendered.contains("classified"));
    assert!(!rendered.contains("hidden content"));
}
