use ruse_mcp::{
    McpPromptArgument, McpPromptDescriptor, McpPromptExport, McpPromptGetParams,
    McpPromptListResult, McpPromptMessage,
};

#[test]
fn mcp_server_prompt_arguments_serialize_stable_descriptor_shape() {
    let descriptor = McpPromptDescriptor::new_with_arguments(
        "review",
        Some("Review current changes"),
        vec![
            McpPromptArgument::new("topic", Some("Review topic"), true).expect("topic argument"),
            McpPromptArgument::new("tone", Some("Optional tone"), false).expect("tone argument"),
        ],
    )
    .expect("descriptor");
    let list = McpPromptListResult {
        prompts: vec![descriptor],
    };

    let value = serde_json::to_value(&list).expect("json");

    assert_eq!(value["prompts"][0]["name"], "review");
    assert_eq!(value["prompts"][0]["arguments"][0]["name"], "topic");
    assert_eq!(
        value["prompts"][0]["arguments"][0]["description"],
        "Review topic"
    );
    assert_eq!(value["prompts"][0]["arguments"][0]["required"], true);
    assert_eq!(value["prompts"][0]["arguments"][1]["name"], "tone");
    assert_eq!(value["prompts"][0]["arguments"][1]["required"], false);
}

#[test]
fn mcp_server_prompt_arguments_get_params_serialize_stable_shape() {
    let params = McpPromptGetParams::new("review")
        .expect("params")
        .with_argument("topic", "workspace diff")
        .expect("topic");

    let value = serde_json::to_value(&params).expect("json");
    let parsed: McpPromptGetParams = serde_json::from_value(value.clone()).expect("params");

    assert_eq!(value["name"], "review");
    assert_eq!(value["arguments"]["topic"], "workspace diff");
    assert_eq!(parsed.name, "review");
    assert_eq!(
        parsed.arguments.get("topic").map(String::as_str),
        Some("workspace diff")
    );
}

#[test]
fn mcp_server_prompt_arguments_export_descriptor_includes_arguments() {
    let export = McpPromptExport::new_with_arguments(
        "review",
        Some("Review current changes"),
        vec![McpPromptArgument::new("topic", Some("Review topic"), true).expect("argument")],
        vec![McpPromptMessage::text("user", "Review {{topic}}").expect("message")],
        true,
    )
    .expect("export");
    let value = serde_json::to_value(export.descriptor()).expect("descriptor json");

    assert_eq!(value["arguments"][0]["name"], "topic");
    assert_eq!(value["arguments"][0]["required"], true);
}

#[test]
fn mcp_server_prompt_arguments_reject_invalid_metadata_safely() {
    let empty_name =
        McpPromptArgument::new(" ", Some("Review topic"), true).expect_err("empty name");
    let secret_name =
        McpPromptArgument::new("api_key", Some("Review topic"), true).expect_err("secret name");
    let empty_description =
        McpPromptArgument::new("topic", Some(" "), true).expect_err("empty description");
    let duplicate = McpPromptDescriptor::new_with_arguments(
        "review",
        Some("Review current changes"),
        vec![
            McpPromptArgument::new("topic", Some("Review topic"), true).expect("first"),
            McpPromptArgument::new("topic", Some("Duplicate topic"), false).expect("second"),
        ],
    )
    .expect_err("duplicate");

    assert!(
        empty_name
            .to_string()
            .contains("invalid MCP prompt argument name")
    );
    assert!(secret_name.to_string().contains("[redacted]"));
    assert!(!secret_name.to_string().contains("api_key"));
    assert!(
        empty_description
            .to_string()
            .contains("invalid MCP prompt argument description")
    );
    assert!(
        duplicate
            .to_string()
            .contains("duplicate MCP prompt argument")
    );
}
