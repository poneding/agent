use agent_mcp::{McpResourceExportRegistry, McpResourceTemplateCompletionSource};
use serde_json::json;

fn workspace_file_registry(enabled: bool) -> McpResourceExportRegistry {
    let mut registry = McpResourceExportRegistry::default();
    if enabled {
        registry
            .export_resource_template_with_completions(
                "agent://workspace/file/{path}",
                "Workspace file",
                Some("Read a workspace file"),
                Some("text/plain"),
                vec![
                    McpResourceTemplateCompletionSource::new(
                        "path",
                        ["README.md", "src/lib.rs", "src/main.rs", "src/lib.rs"],
                    )
                    .expect("path completions"),
                ],
            )
            .expect("template");
    } else {
        registry
            .export_disabled_resource_template_with_completions(
                "agent://workspace/file/{path}",
                "Workspace file",
                Some("Read a workspace file"),
                Some("text/plain"),
                vec![
                    McpResourceTemplateCompletionSource::new("path", ["README.md"])
                        .expect("path completions"),
                ],
            )
            .expect("disabled template");
    }
    registry
}

#[test]
fn mcp_server_resource_template_completions_serialize_source_shape() {
    let source = McpResourceTemplateCompletionSource::new("path", ["src/lib.rs", "README.md"])
        .expect("source");

    let value = serde_json::to_value(&source).expect("json");
    let parsed: McpResourceTemplateCompletionSource =
        serde_json::from_value(value.clone()).expect("source");

    assert_eq!(
        value,
        json!({
            "variable": "path",
            "values": ["README.md", "src/lib.rs"]
        })
    );
    assert_eq!(parsed, source);
}

#[test]
fn mcp_server_resource_template_completions_complete_declared_variable_values() {
    let registry = workspace_file_registry(true);

    let result = registry
        .complete_resource_template_variable("agent://workspace/file/{path}", "path", "src")
        .expect("completion");

    assert_eq!(result.completion.values, vec!["src/lib.rs", "src/main.rs"]);
    assert_eq!(result.completion.total, 2);
    assert!(!result.completion.has_more);
}

#[test]
fn mcp_server_resource_template_completions_validate_declared_variables() {
    let mut registry = McpResourceExportRegistry::default();

    let error = registry
        .export_resource_template_with_completions(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
            vec![
                McpResourceTemplateCompletionSource::new("missing", ["README.md"])
                    .expect("missing source"),
            ],
        )
        .expect_err("undeclared variable");

    assert!(
        error
            .to_string()
            .contains("undeclared MCP resource template completion variable")
    );
}

#[test]
fn mcp_server_resource_template_completions_hide_disabled_templates() {
    let registry = workspace_file_registry(false);

    let error = registry
        .complete_resource_template_variable("agent://workspace/file/{path}", "path", "")
        .expect_err("disabled template");

    assert!(
        error
            .to_string()
            .contains("MCP resource template export not found")
    );
}

#[test]
fn mcp_server_resource_template_completions_return_empty_for_variable_without_source() {
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_resource_template_with_completions(
            "agent://workspace/file/{path}/{name}",
            "Workspace file",
            None::<String>,
            None::<String>,
            vec![McpResourceTemplateCompletionSource::new("path", ["src"]).expect("path source")],
        )
        .expect("template");

    let result = registry
        .complete_resource_template_variable("agent://workspace/file/{path}/{name}", "name", "")
        .expect("completion");

    assert!(result.completion.values.is_empty());
    assert_eq!(result.completion.total, 0);
    assert!(!result.completion.has_more);
}

#[test]
fn mcp_server_resource_template_completions_reject_secret_like_sources_safely() {
    let secret_source = McpResourceTemplateCompletionSource::new("path", ["sk-secret"])
        .expect_err("secret completion");
    let secret_variable = McpResourceTemplateCompletionSource::new("api_key", ["README.md"])
        .expect_err("secret variable");

    assert!(secret_source.to_string().contains("[redacted]"));
    assert!(!secret_source.to_string().contains("sk-secret"));
    assert!(secret_variable.to_string().contains("[redacted]"));
    assert!(!secret_variable.to_string().contains("api_key"));
}
