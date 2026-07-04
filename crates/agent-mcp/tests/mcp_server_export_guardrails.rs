use agent_mcp::{
    McpServerExportPolicyPreviewDecision, McpServerToolExportCapability,
    build_mcp_server_export_policy_preview, inspect_mcp_server_export_profile,
    parse_mcp_server_export_manifest_json,
};
use std::path::PathBuf;

#[test]
fn mcp_server_export_guardrails_allow_read_only_exports() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [
                {"name": "workspace.read_file", "capability": "read_only"},
                {"name": "workspace.search_text"}
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let preview = build_mcp_server_export_policy_preview(&manifest, None).expect("policy preview");

    assert_eq!(preview.tools.len(), 2);
    assert!(preview.tools.iter().all(|tool| {
        tool.capability == McpServerToolExportCapability::ReadOnly
            && tool.decision == McpServerExportPolicyPreviewDecision::Allowed
    }));
    assert!(preview.diagnostics.is_empty());
}

#[test]
fn mcp_server_export_guardrails_reject_enabled_side_effecting_capabilities() {
    for capability in [
        "command",
        "edit",
        "git_write",
        "browser",
        "secret",
        "network",
    ] {
        let manifest = format!(
            r#"{{
              "schema_version": 1,
              "profiles": [
                {{
                  "name": "default",
                  "tools": [
                    {{"name": "workspace.preview", "capability": "{capability}"}}
                  ]
                }}
              ]
            }}"#
        );

        let error = parse_mcp_server_export_manifest_json(&manifest)
            .expect_err("side-effecting capability rejected");
        let message = error.to_string();
        assert!(message.contains("denied by default"));
        assert!(message.contains(capability));
    }
}

#[test]
fn mcp_server_export_guardrails_preview_disabled_side_effecting_capability() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [
                {"name": "workspace.shell_command", "capability": "command", "enabled": false}
              ]
            }
          ]
        }"#,
    )
    .expect("disabled side-effecting manifest");

    let preview = build_mcp_server_export_policy_preview(&manifest, None).expect("policy preview");

    assert_eq!(preview.tools.len(), 1);
    assert_eq!(
        preview.tools[0].decision,
        McpServerExportPolicyPreviewDecision::Denied
    );
    assert_eq!(
        preview.tools[0].capability,
        McpServerToolExportCapability::Command
    );
    assert!(!preview.tools[0].enabled);
    assert!(
        preview
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "tool_export_side_effecting_denied")
    );
}

#[test]
fn mcp_server_export_guardrails_inspection_reports_capabilities() {
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [
                {"name": "workspace.shell_command", "capability": "command", "enabled": false}
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let inspection = inspect_mcp_server_export_profile(
        &manifest,
        agent_mcp::McpServerProfileInspectionOptions {
            profile_name: None,
            manifest_dir: PathBuf::from("."),
            workspace_root: None,
        },
    )
    .expect("inspection");

    assert_eq!(
        inspection.tools[0].capability,
        McpServerToolExportCapability::Command
    );
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "tool_export_side_effecting_denied")
    );
}

#[test]
fn mcp_server_export_guardrails_redact_secret_like_tool_names() {
    let error = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [
                {"name": "workspace.sk-secret-token", "capability": "command"}
              ]
            }
          ]
        }"#,
    )
    .expect_err("secret tool rejected");

    let message = error.to_string();
    assert!(message.contains("[redacted]"));
    assert!(!message.contains("sk-secret-token"));
}
