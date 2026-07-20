#[test]
fn mcp_policy_mapping_normalizes_local_tool_name() {
    let server_id = ruse_mcp::McpServerId::new("filesystem").unwrap();
    let name = ruse_mcp::local_tool_name_for_mcp_tool(&server_id, "Read File").unwrap();
    assert_eq!(name.as_str(), "mcp.filesystem.read_file");
}

#[test]
fn mcp_policy_mapping_converts_descriptor_to_tool_definition() {
    let server_id = ruse_mcp::McpServerId::new("filesystem").unwrap();
    let descriptor = ruse_mcp::McpToolDescriptor::new(
        server_id,
        "read_file",
        Some("Read File"),
        "Read a file",
        serde_json::json!({"type": "object"}),
    )
    .unwrap();

    let mapped = ruse_mcp::mcp_descriptor_to_tool_definition(&descriptor).unwrap();

    assert_eq!(mapped.local_name.as_str(), "mcp.filesystem.read_file");
    assert_eq!(mapped.definition.name.as_str(), "mcp.filesystem.read_file");
    assert_eq!(mapped.definition.description, "Read a file");
    assert_eq!(
        mapped.definition.input_schema,
        serde_json::json!({"type": "object"})
    );
}

#[test]
fn mcp_policy_mapping_builds_default_permission_request() {
    let actor = ruse_types::ActorRef::system();
    let server_id = ruse_mcp::McpServerId::new("filesystem").unwrap();
    let descriptor = ruse_mcp::McpToolDescriptor::new(
        server_id,
        "read_file",
        None::<String>,
        "Read a file",
        serde_json::json!({"type": "object"}),
    )
    .unwrap();

    let mapping = ruse_mcp::default_policy_mapping_for_mcp_tool(&descriptor).unwrap();
    let request = ruse_mcp::permission_request_for_mcp_mapping(&mapping, actor.clone());

    assert_eq!(request.actor, actor);
    assert_eq!(request.action, ruse_policy::CapabilityAction::Execute);
    assert_eq!(request.scope.kind, ruse_policy::ResourceKind::Tool);
    assert_eq!(request.scope.pattern, "mcp:filesystem:read_file");
}

#[test]
fn mcp_policy_mapping_rejects_empty_remote_name() {
    let server_id = ruse_mcp::McpServerId::new("filesystem").unwrap();
    let result = ruse_mcp::local_tool_name_for_mcp_tool(&server_id, " ");
    assert!(result.is_err());
}
