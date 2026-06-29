use agent_mcp::{
    MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION, McpServerExportManifest,
    McpServerExportProfileConfig, McpServerProfileInspectionOptions, McpServerPromptArgumentConfig,
    McpServerPromptExportConfig, McpServerPromptMessageConfig,
    build_mcp_server_profile_from_manifest, inspect_mcp_server_export_profile,
    parse_mcp_server_export_manifest_json,
};
use std::collections::BTreeMap;

#[test]
fn mcp_server_manifest_prompt_arguments_serialize_and_parse_stable_shape() {
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
                arguments: vec![McpServerPromptArgumentConfig {
                    name: "topic".to_string(),
                    description: Some("Review topic".to_string()),
                    required: true,
                }],
                messages: vec![McpServerPromptMessageConfig {
                    role: "user".to_string(),
                    text: "Review {{topic}}".to_string(),
                }],
                enabled: true,
            }],
        }],
    };

    let value = serde_json::to_value(&manifest).expect("manifest json");
    let parsed = parse_mcp_server_export_manifest_json(&value.to_string()).expect("parse");

    assert_eq!(
        value["profiles"][0]["prompts"][0]["arguments"][0]["name"],
        "topic"
    );
    assert_eq!(
        value["profiles"][0]["prompts"][0]["arguments"][0]["description"],
        "Review topic"
    );
    assert_eq!(parsed.profiles[0].prompts[0].arguments[0].name, "topic");
    assert!(parsed.profiles[0].prompts[0].arguments[0].required);
}

#[test]
fn mcp_server_manifest_prompt_arguments_build_runtime_renderer() {
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
                  "arguments": [
                    {"name": "topic", "description": "Review topic", "required": true}
                  ],
                  "messages": [{"role": "user", "text": "Review {{topic}}"}]
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
    let mut arguments = BTreeMap::new();
    arguments.insert("topic".to_string(), "workspace diff".to_string());

    let result = runtime
        .prompt_exports()
        .get_prompt_with_arguments("review", &arguments)
        .expect("render");

    assert_eq!(result.messages[0].content.text, "Review workspace diff");
}

#[test]
fn mcp_server_manifest_prompt_arguments_reject_duplicates() {
    let duplicate = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "arguments": [
                    {"name": "topic", "description": "Review topic", "required": true},
                    {"name": "topic", "description": "Duplicate topic", "required": false}
                  ],
                  "messages": [{"role": "user", "text": "Review {{topic}}"}]
                }
              ]
            }
          ]
        }"#,
    )
    .expect_err("duplicate argument");

    assert!(duplicate.to_string().contains("duplicate"));
    assert!(duplicate.to_string().contains("prompt argument"));
}

#[test]
fn mcp_server_manifest_prompt_arguments_inspection_summarizes_without_contents() {
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
                  "arguments": [
                    {"name": "topic", "description": "Review topic", "required": true},
                    {"name": "tone", "description": "Optional tone", "required": false}
                  ],
                  "messages": [{"role": "user", "text": "classified {{topic}} content"}]
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

    assert_eq!(inspection.prompts[0].argument_count, 2);
    assert_eq!(inspection.prompts[0].arguments[0].name, "topic");
    assert_eq!(
        inspection.prompts[0].arguments[0].description,
        Some("Review topic".to_string())
    );
    assert!(inspection.prompts[0].arguments[0].required);
    assert_eq!(inspection.prompts[0].arguments[1].name, "tone");
    assert!(!inspection.prompts[0].arguments[1].required);
    assert!(!rendered.contains("classified"));
}
