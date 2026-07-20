use ruse_mcp::{
    McpPromptArgument, McpPromptArgumentCompletionSource, McpPromptExport, McpPromptExportRegistry,
    McpPromptMessage,
};
use serde_json::json;

fn review_prompt(enabled: bool) -> McpPromptExport {
    McpPromptExport::new_with_argument_completions(
        "review",
        Some("Review current changes"),
        vec![
            McpPromptArgument::new("topic", Some("Review topic"), true).expect("topic"),
            McpPromptArgument::new("tone", Some("Review tone"), false).expect("tone"),
            McpPromptArgument::new("scope", Some("Review scope"), false).expect("scope"),
        ],
        vec![
            McpPromptArgumentCompletionSource::new(
                "topic",
                [
                    "workspace diff",
                    "worktree status",
                    "workspace diff",
                    "release notes",
                ],
            )
            .expect("topic completions"),
            McpPromptArgumentCompletionSource::new("tone", ["concise", "careful"])
                .expect("tone completions"),
        ],
        vec![McpPromptMessage::text("user", "Review {{topic}}").expect("message")],
        enabled,
    )
    .expect("prompt")
}

#[test]
fn mcp_server_prompt_completions_serialize_source_shape() {
    let source =
        McpPromptArgumentCompletionSource::new("topic", ["workspace diff", "release notes"])
            .expect("source");

    let value = serde_json::to_value(&source).expect("json");
    let parsed: McpPromptArgumentCompletionSource =
        serde_json::from_value(value.clone()).expect("source");

    assert_eq!(
        value,
        json!({
            "argument": "topic",
            "values": ["release notes", "workspace diff"]
        })
    );
    assert_eq!(parsed, source);
}

#[test]
fn mcp_server_prompt_completions_complete_declared_argument_values() {
    let mut registry = McpPromptExportRegistry::default();
    registry
        .export_prompt(review_prompt(true))
        .expect("export prompt");

    let result = registry
        .complete_prompt_argument("review", "topic", "work")
        .expect("completion");

    assert_eq!(
        result.completion.values,
        vec!["workspace diff", "worktree status"]
    );
    assert_eq!(result.completion.total, 2);
    assert!(!result.completion.has_more);
}

#[test]
fn mcp_server_prompt_completions_validate_declared_arguments() {
    let undeclared = McpPromptExport::new_with_argument_completions(
        "review",
        Some("Review current changes"),
        vec![McpPromptArgument::new("topic", Some("Review topic"), true).expect("topic")],
        vec![McpPromptArgumentCompletionSource::new("tone", ["concise"]).expect("tone completion")],
        vec![McpPromptMessage::text("user", "Review {{topic}}").expect("message")],
        true,
    )
    .expect_err("undeclared completion argument");

    assert!(
        undeclared
            .to_string()
            .contains("undeclared MCP prompt completion argument")
    );
}

#[test]
fn mcp_server_prompt_completions_hide_disabled_prompts() {
    let mut registry = McpPromptExportRegistry::default();
    registry
        .export_prompt(review_prompt(false))
        .expect("export disabled prompt");

    let error = registry
        .complete_prompt_argument("review", "topic", "work")
        .expect_err("disabled prompt");

    assert!(error.to_string().contains("MCP prompt export not found"));
}

#[test]
fn mcp_server_prompt_completions_return_empty_for_argument_without_source() {
    let mut registry = McpPromptExportRegistry::default();
    registry
        .export_prompt(review_prompt(true))
        .expect("export prompt");

    let result = registry
        .complete_prompt_argument("review", "scope", "")
        .expect("completion");

    assert!(result.completion.values.is_empty());
    assert_eq!(result.completion.total, 0);
    assert!(!result.completion.has_more);
}

#[test]
fn mcp_server_prompt_completions_reject_secret_like_sources_safely() {
    let secret_source = McpPromptArgumentCompletionSource::new("topic", ["sk-secret"])
        .expect_err("secret completion");
    let secret_argument = McpPromptArgumentCompletionSource::new("api_key", ["workspace"])
        .expect_err("secret argument");

    assert!(secret_source.to_string().contains("[redacted]"));
    assert!(!secret_source.to_string().contains("sk-secret"));
    assert!(secret_argument.to_string().contains("[redacted]"));
    assert!(!secret_argument.to_string().contains("api_key"));
}
