use agent_mcp::{
    McpCompletionArgument, McpCompletionRef, McpCompletionRequest, McpCompletionResult,
};
use serde_json::json;

#[test]
fn mcp_server_completion_contracts_serialize_prompt_request_shape() {
    let request = McpCompletionRequest::new(
        McpCompletionRef::prompt("review").expect("prompt ref"),
        McpCompletionArgument::new("topic", "work").expect("argument"),
    );

    let value = serde_json::to_value(&request).expect("json");
    let parsed: McpCompletionRequest =
        serde_json::from_value(value.clone()).expect("completion request");

    assert_eq!(
        value,
        json!({
            "ref": {
                "type": "ref/prompt",
                "name": "review"
            },
            "argument": {
                "name": "topic",
                "value": "work"
            }
        })
    );
    assert_eq!(parsed.completion_ref, request.completion_ref);
    assert_eq!(parsed.argument, request.argument);
}

#[test]
fn mcp_server_completion_contracts_serialize_resource_request_shape() {
    let request = McpCompletionRequest::new(
        McpCompletionRef::resource("agent://workspace/file/{path}").expect("resource ref"),
        McpCompletionArgument::new("path", "READ").expect("argument"),
    );

    let value = serde_json::to_value(&request).expect("json");

    assert_eq!(value["ref"]["type"], "ref/resource");
    assert_eq!(value["ref"]["uri"], "agent://workspace/file/{path}");
    assert_eq!(value["argument"]["name"], "path");
    assert_eq!(value["argument"]["value"], "READ");
}

#[test]
fn mcp_server_completion_contracts_builds_bounded_protocol_result() {
    let result = McpCompletionResult::from_values(
        "wo",
        [
            "workspace",
            "worktree",
            "workspace",
            "alpha",
            "workflow",
            "world",
        ],
        3,
    )
    .expect("completion result");
    let value = serde_json::to_value(&result).expect("json");

    assert_eq!(
        value,
        json!({
            "completion": {
                "values": ["workflow", "workspace", "worktree"],
                "total": 4,
                "hasMore": true
            }
        })
    );
}

#[test]
fn mcp_server_completion_contracts_allow_empty_prefix_and_empty_results() {
    let result =
        McpCompletionResult::from_values("", ["beta", "alpha"], 10).expect("empty prefix result");
    let empty = McpCompletionResult::from_values("z", ["beta", "alpha"], 10).expect("empty result");

    assert_eq!(result.completion.values, vec!["alpha", "beta"]);
    assert_eq!(result.completion.total, 2);
    assert!(!result.completion.has_more);
    assert!(empty.completion.values.is_empty());
    assert_eq!(empty.completion.total, 0);
    assert!(!empty.completion.has_more);
}

#[test]
fn mcp_server_completion_contracts_reject_invalid_metadata_safely() {
    let empty_argument = McpCompletionArgument::new(" ", "work").expect_err("empty argument name");
    let secret_argument =
        McpCompletionArgument::new("api_key", "work").expect_err("secret argument name");
    let secret_prefix =
        McpCompletionArgument::new("topic", "sk-secret").expect_err("secret prefix");
    let secret_ref = McpCompletionRef::prompt("token").expect_err("secret prompt ref");
    let secret_value = McpCompletionResult::from_values("", ["sk-secret"], 10)
        .expect_err("secret completion value");

    assert!(
        empty_argument
            .to_string()
            .contains("invalid MCP completion argument name")
    );
    assert!(secret_argument.to_string().contains("[redacted]"));
    assert!(!secret_argument.to_string().contains("api_key"));
    assert!(secret_prefix.to_string().contains("[redacted]"));
    assert!(!secret_prefix.to_string().contains("sk-secret"));
    assert!(secret_ref.to_string().contains("[redacted]"));
    assert!(!secret_ref.to_string().contains("token"));
    assert!(secret_value.to_string().contains("[redacted]"));
    assert!(!secret_value.to_string().contains("sk-secret"));
}
