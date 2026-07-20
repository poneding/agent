use ruse_mcp::{
    McpServerToolExportCatalog, build_mcp_server_profile_from_manifest,
    inspect_mcp_server_export_profile, parse_mcp_server_export_manifest_json,
};
use tempfile::TempDir;

#[test]
fn mcp_server_manifest_completions_builds_prompt_argument_sources() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "arguments": [
                    {"name": "topic", "description": "Review topic", "required": true}
                  ],
                  "argument_completions": [
                    {"argument": "topic", "values": ["workspace diff", "worktree status"]}
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
        &McpServerToolExportCatalog::default(),
    )
    .expect("runtime");

    let result = runtime
        .prompt_exports()
        .complete_prompt_argument("review", "topic", "work")
        .expect("completion");

    assert_eq!(
        result.completion.values,
        vec!["workspace diff", "worktree status"]
    );
}

#[test]
fn mcp_server_manifest_completions_builds_resource_template_sources() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "hello").expect("resource");
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
                  "variable_completions": [
                    {"variable": "path", "values": ["README.md", "src/lib.rs"]}
                  ]
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
        temp.path(),
        &McpServerToolExportCatalog::default(),
    )
    .expect("runtime");

    let result = runtime
        .resource_exports()
        .complete_resource_template_variable("agent://workspace/file/{path}", "path", "READ")
        .expect("completion");

    assert_eq!(result.completion.values, vec!["README.md"]);
}

#[test]
fn mcp_server_manifest_completions_inspection_summarizes_completion_counts() {
    let temp = TempDir::new().expect("tempdir");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "arguments": [{"name": "topic"}],
                  "argument_completions": [
                    {"argument": "topic", "values": ["workspace diff"]}
                  ],
                  "messages": [{"role": "user", "text": "Review {{topic}}"}]
                }
              ],
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "path_template": "{path}",
                  "variable_completions": [
                    {"variable": "path", "values": ["README.md"]}
                  ]
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
    let rendered = serde_json::to_string(&inspection).expect("json");

    assert_eq!(inspection.prompts[0].completion_count, 1);
    assert_eq!(inspection.resource_templates[0].completion_count, 1);
    assert!(!rendered.contains("workspace diff"));
    assert!(!rendered.contains("README.md"));
}

#[test]
fn mcp_server_manifest_completions_reject_invalid_sources_safely() {
    let error = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "arguments": [{"name": "topic"}],
                  "argument_completions": [
                    {"argument": "topic", "values": ["sk-secret"]}
                  ],
                  "messages": [{"role": "user", "text": "Review {{topic}}"}]
                }
              ]
            }
          ]
        }"#,
    )
    .expect_err("secret completion");

    assert!(error.to_string().contains("[redacted]"));
    assert!(!error.to_string().contains("sk-secret"));
}
