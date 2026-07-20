use ruse_types::{
    McpPolicyMappingRecorded, McpToolsDiscovered, RuntimeEventPayload,
    SkillPromptContributionRecorded,
};
use serde_json::json;

#[test]
fn m07_external_trace_serializes_mcp_tools_discovered() {
    let payload = RuntimeEventPayload::McpToolsDiscovered(McpToolsDiscovered {
        server_id: "filesystem".to_string(),
        tool_count: 1,
        local_tool_names: vec!["mcp.filesystem.read_file".to_string()],
    });

    let value = serde_json::to_value(payload).expect("serialize MCP discovery payload");

    assert_eq!(
        value,
        json!({
            "type": "mcp_tools_discovered",
            "data": {
                "server_id": "filesystem",
                "tool_count": 1,
                "local_tool_names": ["mcp.filesystem.read_file"]
            }
        })
    );
}

#[test]
fn m07_external_trace_serializes_skill_prompt_contribution_without_body() {
    let payload =
        RuntimeEventPayload::SkillPromptContributionRecorded(SkillPromptContributionRecorded {
            skill_id: "code-review".to_string(),
            layer_id: "skill:code-review".to_string(),
            priority: 600,
            token_estimate: 4,
            source_path: "/repo/.codex/skills/code-review/SKILL.md".to_string(),
        });

    let serialized = serde_json::to_string(&payload).expect("serialize skill trace payload");

    assert!(serialized.contains("skill_prompt_contribution_recorded"));
    assert!(serialized.contains("code-review"));
    assert!(!serialized.contains("Check diffs carefully."));
}

#[test]
fn m07_external_trace_serializes_mcp_policy_mapping() {
    let payload = RuntimeEventPayload::McpPolicyMappingRecorded(McpPolicyMappingRecorded {
        server_id: "filesystem".to_string(),
        remote_tool_name: "read_file".to_string(),
        local_tool_name: "mcp.filesystem.read_file".to_string(),
        action: "execute".to_string(),
        resource_kind: "tool".to_string(),
        resource_pattern: "mcp:filesystem:read_file".to_string(),
    });

    let value = serde_json::to_value(payload).expect("serialize MCP policy trace payload");

    assert_eq!(value["type"], "mcp_policy_mapping_recorded");
    assert_eq!(value["data"]["action"], "execute");
    assert_eq!(value["data"]["resource_kind"], "tool");
    assert_eq!(
        value["data"]["resource_pattern"],
        "mcp:filesystem:read_file"
    );
}
