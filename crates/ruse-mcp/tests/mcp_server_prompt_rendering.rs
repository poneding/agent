use std::collections::BTreeMap;

use ruse_mcp::{McpPromptArgument, McpPromptExport, McpPromptExportRegistry, McpPromptMessage};

fn arguments(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
        .collect()
}

fn review_prompt() -> McpPromptExport {
    McpPromptExport::new_with_arguments(
        "review",
        Some("Review current changes"),
        vec![
            McpPromptArgument::new("topic", Some("Review topic"), true).expect("topic"),
            McpPromptArgument::new("tone", Some("Optional tone"), false).expect("tone"),
        ],
        vec![
            McpPromptMessage::text("system", "Use a {{tone}} tone.").expect("system"),
            McpPromptMessage::text("user", "Review {{topic}}.").expect("user"),
        ],
        true,
    )
    .expect("prompt")
}

#[test]
fn mcp_server_prompt_rendering_substitutes_declared_arguments() {
    let result = review_prompt()
        .get_result_with_arguments(&arguments(&[
            ("topic", "workspace diff"),
            ("tone", "concise"),
        ]))
        .expect("render");

    assert_eq!(result.messages[0].content.text, "Use a concise tone.");
    assert_eq!(result.messages[1].content.text, "Review workspace diff.");
}

#[test]
fn mcp_server_prompt_rendering_handles_optional_missing_as_empty() {
    let result = review_prompt()
        .get_result_with_arguments(&arguments(&[("topic", "workspace diff")]))
        .expect("render");

    assert_eq!(result.messages[0].content.text, "Use a  tone.");
    assert_eq!(result.messages[1].content.text, "Review workspace diff.");
}

#[test]
fn mcp_server_prompt_rendering_rejects_missing_and_unknown_arguments() {
    let mut registry = McpPromptExportRegistry::default();
    registry.export_prompt(review_prompt()).expect("export");

    let missing = registry
        .get_prompt_with_arguments("review", &BTreeMap::new())
        .expect_err("missing required");
    let unknown = registry
        .get_prompt_with_arguments(
            "review",
            &arguments(&[("topic", "workspace diff"), ("extra", "unexpected")]),
        )
        .expect_err("unknown argument");

    assert!(
        missing
            .to_string()
            .contains("missing required MCP prompt argument")
    );
    assert!(unknown.to_string().contains("unknown MCP prompt argument"));
}

#[test]
fn mcp_server_prompt_rendering_rejects_undeclared_template_references() {
    let undeclared = McpPromptExport::new_with_arguments(
        "review",
        Some("Review current changes"),
        vec![McpPromptArgument::new("topic", Some("Review topic"), true).expect("topic")],
        vec![McpPromptMessage::text("user", "Review {{missing}}.").expect("message")],
        true,
    )
    .expect_err("undeclared reference");
    let unclosed = McpPromptExport::new_with_arguments(
        "review",
        Some("Review current changes"),
        vec![McpPromptArgument::new("topic", Some("Review topic"), true).expect("topic")],
        vec![McpPromptMessage::text("user", "Review {{topic.").expect("message")],
        true,
    )
    .expect_err("unclosed reference");

    assert!(
        undeclared
            .to_string()
            .contains("undeclared MCP prompt argument")
    );
    assert!(
        unclosed
            .to_string()
            .contains("unclosed MCP prompt template reference")
    );
}

#[test]
fn mcp_server_prompt_rendering_redacts_and_bounds_argument_values() {
    let secret = review_prompt()
        .get_result_with_arguments(&arguments(&[("topic", "api_key sk-secret-value")]))
        .expect("render secret");
    let long_value = format!("{}{}", "a".repeat(4096), "tail");
    let long = review_prompt()
        .get_result_with_arguments(&arguments(&[("topic", &long_value)]))
        .expect("render long");

    assert_eq!(secret.messages[1].content.text, "Review [redacted].");
    assert!(!secret.messages[1].content.text.contains("sk-secret-value"));
    assert!(long.messages[1].content.text.len() < 1100);
    assert!(long.messages[1].content.text.contains("..."));
    assert!(!long.messages[1].content.text.contains("tail"));
}
