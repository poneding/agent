use ruse_mcp::{McpResourceExportRegistry, McpResourceTemplateExport};
use serde_json::json;

#[test]
fn mcp_server_resource_template_contracts_serialize_stable_descriptor_shape() {
    let template = McpResourceTemplateExport {
        uri_template: "agent://workspace/file/{path}".to_string(),
        name: "Workspace file".to_string(),
        description: Some("Read a workspace file".to_string()),
        mime_type: Some("text/plain".to_string()),
        enabled: true,
    };

    let value = serde_json::to_value(template).expect("template json");

    assert_eq!(
        value,
        json!({
            "uriTemplate": "agent://workspace/file/{path}",
            "name": "Workspace file",
            "description": "Read a workspace file",
            "mimeType": "text/plain",
            "enabled": true
        })
    );
}

#[test]
fn mcp_server_resource_template_contracts_list_enabled_templates_in_order() {
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_resource_template(
            "agent://workspace/z/{path}",
            "Zed",
            None::<String>,
            None::<String>,
        )
        .expect("z template");
    registry
        .export_disabled_resource_template(
            "agent://workspace/a/{path}",
            "Hidden",
            Some("Hidden template"),
            Some("text/plain"),
        )
        .expect("disabled template");
    registry
        .export_resource_template(
            "agent://workspace/b/{path}",
            "Beta",
            Some("Beta template"),
            Some("text/markdown"),
        )
        .expect("beta template");

    let listed = registry.list_resource_templates();

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].uri_template, "agent://workspace/b/{path}");
    assert_eq!(listed[0].description.as_deref(), Some("Beta template"));
    assert_eq!(listed[0].mime_type.as_deref(), Some("text/markdown"));
    assert_eq!(listed[1].uri_template, "agent://workspace/z/{path}");
}

#[test]
fn mcp_server_resource_template_contracts_reject_duplicate_templates() {
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
        )
        .expect("template");

    let error = registry
        .export_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file copy",
            None::<String>,
            None::<String>,
        )
        .expect_err("duplicate template");

    assert!(
        error
            .to_string()
            .contains("duplicate MCP resource template export")
    );
}

#[test]
fn mcp_server_resource_template_contracts_reject_invalid_metadata_safely() {
    let mut registry = McpResourceExportRegistry::default();

    let invalid_uri = registry
        .export_resource_template("not-a-uri/{path}", "Bad", None::<String>, None::<String>)
        .expect_err("invalid uri");
    assert!(
        invalid_uri
            .to_string()
            .contains("invalid MCP resource template")
    );

    let missing_variable = registry
        .export_resource_template(
            "agent://workspace/file/path",
            "Bad",
            None::<String>,
            None::<String>,
        )
        .expect_err("missing variable");
    assert!(
        missing_variable
            .to_string()
            .contains("invalid MCP resource template")
    );

    let secret_description = registry
        .export_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            Some("api_key backed files"),
            None::<String>,
        )
        .expect_err("secret description");
    assert!(!secret_description.to_string().contains("api_key"));
}
