use agent_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId,
    McpJsonRpcRequest, McpJsonRpcSuccess, McpServerCapabilities, McpServerInfo, McpServerMethod,
    McpServerState,
};

#[test]
fn mcp_server_contracts_info_serializes_stable_shape() {
    let state = McpServerState::new(
        McpServerInfo::new("agent", "1.2.3", MCP_SERVER_PROTOCOL_VERSION).unwrap(),
        McpServerCapabilities {
            tools: true,
            prompts: false,
            resources: true,
        },
    );

    let value = serde_json::to_value(&state).expect("server state json");

    assert_eq!(value["info"]["name"], "agent");
    assert_eq!(value["info"]["version"], "1.2.3");
    assert_eq!(
        value["info"]["protocol_version"],
        MCP_SERVER_PROTOCOL_VERSION
    );
    assert_eq!(value["capabilities"]["tools"], true);
    assert_eq!(value["capabilities"]["prompts"], false);
    assert_eq!(value["capabilities"]["resources"], true);
    assert_eq!(value["initialized"], false);
}

#[test]
fn mcp_server_contracts_info_rejects_empty_or_secret_like_metadata() {
    assert!(McpServerInfo::new(" ", "1.0.0", MCP_SERVER_PROTOCOL_VERSION).is_err());
    assert!(McpServerInfo::new("agent", " ", MCP_SERVER_PROTOCOL_VERSION).is_err());
    assert!(McpServerInfo::new("agent", "1.0.0", " ").is_err());
    assert!(McpServerInfo::new("api_key", "1.0.0", MCP_SERVER_PROTOCOL_VERSION).is_err());
    assert!(McpServerInfo::new("agent", "sk-secret", MCP_SERVER_PROTOCOL_VERSION).is_err());
}

#[test]
fn mcp_server_contracts_method_parses_supported_wire_names() {
    assert_eq!(
        McpServerMethod::parse("initialize").unwrap(),
        McpServerMethod::Initialize
    );
    assert_eq!(
        McpServerMethod::parse("tools/list").unwrap(),
        McpServerMethod::ToolsList
    );
    assert_eq!(
        McpServerMethod::parse("tools/call").unwrap(),
        McpServerMethod::ToolsCall
    );
    assert_eq!(
        McpServerMethod::parse("resources/list").unwrap(),
        McpServerMethod::ResourcesList
    );
    assert_eq!(
        McpServerMethod::parse("resources/templates/list").unwrap(),
        McpServerMethod::ResourcesTemplatesList
    );
    assert_eq!(
        McpServerMethod::parse("resources/read").unwrap(),
        McpServerMethod::ResourcesRead
    );
    assert!(McpServerMethod::parse("secrets/read").is_err());
}

#[test]
fn mcp_server_contracts_json_rpc_request_preserves_id_shapes() {
    let string_id = serde_json::from_value::<McpJsonRpcRequest>(serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-1",
        "method": "initialize",
        "params": {"client": "fixture"}
    }))
    .expect("string id request");
    let numeric_id = serde_json::from_value::<McpJsonRpcRequest>(serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "tools/list"
    }))
    .expect("numeric id request");
    let notification = serde_json::from_value::<McpJsonRpcRequest>(serde_json::json!({
        "jsonrpc": "2.0",
        "method": "resources/list"
    }))
    .expect("notification request");

    assert_eq!(
        string_id.id,
        Some(McpJsonRpcId::String("req-1".to_string()))
    );
    assert_eq!(numeric_id.id, Some(McpJsonRpcId::Number(42)));
    assert_eq!(notification.id, None);
    assert_eq!(notification.params, serde_json::json!({}));
}

#[test]
fn mcp_server_contracts_json_rpc_success_and_failure_render_stable_shape() {
    let success = McpJsonRpcSuccess::new(
        McpJsonRpcId::String("req-1".to_string()),
        serde_json::json!({"ok": true}),
    );
    let failure = McpJsonRpcFailure::new(
        McpJsonRpcId::Null,
        McpJsonRpcError::method_not_found("api_key sk-secret {}", "x".repeat(400)),
    );

    let success_json = serde_json::to_value(&success).expect("success json");
    let failure_json = serde_json::to_value(&failure).expect("failure json");

    assert_eq!(success_json["jsonrpc"], "2.0");
    assert_eq!(success_json["id"], "req-1");
    assert_eq!(success_json["result"]["ok"], true);
    assert_eq!(failure_json["jsonrpc"], "2.0");
    assert_eq!(failure_json["id"], serde_json::Value::Null);
    assert_eq!(failure_json["error"]["code"], -32601);
    assert_eq!(failure_json["error"]["message"], "[redacted]");
    assert!(!failure_json.to_string().contains("sk-secret"));
}
