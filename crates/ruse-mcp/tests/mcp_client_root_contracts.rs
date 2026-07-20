use ruse_mcp::{
    MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD, McpClientCapabilities, McpClientRootsCapability,
    McpRoot, McpRootListChangedNotification, McpRootListResult,
};
use serde_json::json;

#[test]
fn mcp_client_root_contracts_serialize_root_list_shape() {
    let result = McpRootListResult::from_roots([
        McpRoot::new("file:///workspace/b", Some("b")).expect("root b"),
        McpRoot::new("file:///workspace/a", Some("a")).expect("root a"),
        McpRoot::new("file:///workspace/unnamed", None::<String>).expect("unnamed root"),
    ]);

    let value = serde_json::to_value(&result).expect("json");
    let parsed: McpRootListResult = serde_json::from_value(value.clone()).expect("root list");

    assert_eq!(
        value,
        json!({
            "roots": [
                {
                    "uri": "file:///workspace/a",
                    "name": "a"
                },
                {
                    "uri": "file:///workspace/b",
                    "name": "b"
                },
                {
                    "uri": "file:///workspace/unnamed"
                }
            ]
        })
    );
    assert_eq!(parsed, result);
}

#[test]
fn mcp_client_root_contracts_serialize_capability_shape() {
    let without_roots = McpClientCapabilities::default();
    let with_roots = McpClientCapabilities::with_roots(McpClientRootsCapability::new(true));

    assert_eq!(
        serde_json::to_value(&without_roots).expect("json"),
        json!({})
    );
    assert_eq!(
        serde_json::to_value(&with_roots).expect("json"),
        json!({
            "roots": {
                "listChanged": true
            }
        })
    );
}

#[test]
fn mcp_client_root_contracts_serialize_list_changed_notification_shape() {
    let notification = McpRootListChangedNotification::new();
    let value = serde_json::to_value(&notification).expect("json");
    let parsed: McpRootListChangedNotification =
        serde_json::from_value(value.clone()).expect("notification");

    assert_eq!(
        value,
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/roots/list_changed"
        })
    );
    assert_eq!(parsed.method, MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD);
}

#[test]
fn mcp_client_root_contracts_reject_invalid_metadata_safely() {
    let empty_uri = McpRoot::new(" ", Some("workspace")).expect_err("empty uri");
    let invalid_uri = McpRoot::new("workspace/project", Some("workspace")).expect_err("bad uri");
    let secret_uri =
        McpRoot::new("file:///tmp/sk-secret", Some("workspace")).expect_err("secret-like uri");
    let empty_name =
        McpRoot::new("file:///workspace/project", Some(" ")).expect_err("empty root name");
    let secret_name = McpRoot::new("file:///workspace/project", Some("api_key"))
        .expect_err("secret-like root name");

    assert!(empty_uri.to_string().contains("invalid MCP root URI"));
    assert!(invalid_uri.to_string().contains("invalid MCP root URI"));
    assert!(secret_uri.to_string().contains("[redacted]"));
    assert!(!secret_uri.to_string().contains("sk-secret"));
    assert!(empty_name.to_string().contains("invalid MCP root name"));
    assert!(secret_name.to_string().contains("[redacted]"));
    assert!(!secret_name.to_string().contains("api_key"));
}
