use ruse_mcp::{
    McpPromptDescriptor, McpPromptExport, McpPromptGetResult, McpPromptListResult, McpPromptMessage,
};

#[test]
fn mcp_server_prompt_export_serializes_stable_shape() {
    let export = McpPromptExport::new(
        "default",
        Some("Default agent prompt"),
        vec![McpPromptMessage::text("user", "Summarize this workspace").expect("message")],
        true,
    )
    .expect("prompt export");
    let list = McpPromptListResult {
        prompts: vec![export.descriptor().clone()],
    };
    let get = export.get_result();

    let list_value = serde_json::to_value(&list).expect("list json");
    let get_value = serde_json::to_value(&get).expect("get json");

    assert_eq!(list_value["prompts"][0]["name"], "default");
    assert_eq!(
        list_value["prompts"][0]["description"],
        "Default agent prompt"
    );
    assert_eq!(get_value["name"], "default");
    assert_eq!(
        get_value["messages"][0]["content"]["text"],
        "Summarize this workspace"
    );
    assert_eq!(get_value["messages"][0]["content"]["type"], "text");
}

#[test]
fn mcp_server_prompt_export_rejects_invalid_contracts() {
    let empty_name = McpPromptExport::new(
        " ",
        Some("Default"),
        vec![McpPromptMessage::text("user", "hello").expect("message")],
        true,
    )
    .expect_err("empty name rejected");
    let secret_name = McpPromptExport::new(
        "api_key",
        Some("Default"),
        vec![McpPromptMessage::text("user", "hello").expect("message")],
        true,
    )
    .expect_err("secret name rejected");
    let empty_messages = McpPromptExport::new("default", Some("Default"), Vec::new(), true)
        .expect_err("empty messages rejected");
    let empty_role = McpPromptMessage::text(" ", "hello").expect_err("empty role rejected");

    assert!(empty_name.to_string().contains("invalid MCP prompt name"));
    assert!(secret_name.to_string().contains("[redacted]"));
    assert!(empty_messages.to_string().contains("messages"));
    assert!(empty_role.to_string().contains("role"));
}

#[test]
fn mcp_server_prompt_export_redacts_and_bounds_message_text() {
    let long_text = format!("{}{}", "a".repeat(4096), "api_key sk-secret-value");
    let export = McpPromptExport::new(
        "default",
        Some("Default"),
        vec![McpPromptMessage::text("user", long_text).expect("message")],
        true,
    )
    .expect("prompt export");
    let rendered = serde_json::to_string(&export.get_result()).expect("json");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-secret-value"));
    assert!(rendered.len() < 1200);
}

#[test]
fn mcp_server_prompt_export_descriptor_and_result_contracts_are_constructible() {
    let descriptor =
        McpPromptDescriptor::new("default", Some("Default prompt")).expect("descriptor");
    let message = McpPromptMessage::text("assistant", "Ready").expect("message");
    let result = McpPromptGetResult::new(descriptor.clone(), vec![message]).expect("get result");

    assert_eq!(descriptor.name, "default");
    assert_eq!(result.name, "default");
    assert_eq!(result.messages.len(), 1);
    assert_eq!(result.messages[0].role, "assistant");
}
