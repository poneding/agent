use ruse_tool::{
    Tool, ToolDefinition, ToolInput, ToolName, ToolOutput, ToolRegistry, ToolRegistryError,
};
use serde_json::json;

struct EchoTool {
    name: ToolName,
}

impl EchoTool {
    fn new(name: &str) -> Self {
        Self {
            name: ToolName::new(name).expect("valid tool name"),
        }
    }
}

impl Tool for EchoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: "echo input".to_string(),
            input_schema: json!({
                "type": "object",
                "additionalProperties": true
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ruse_tool::ToolError> {
        Ok(ToolOutput { value: input.value })
    }
}

#[test]
fn tool_name_accepts_registry_safe_names() {
    let name = ToolName::new("workspace.read_file").expect("valid tool name");

    assert_eq!(name.as_str(), "workspace.read_file");
}

#[test]
fn tool_name_rejects_invalid_names() {
    for value in ["", "ReadFile", "read file", "read/file", " read_file"] {
        assert!(ToolName::new(value).is_err(), "{value} should be invalid");
    }
}

#[test]
fn registry_registers_and_executes_tool() {
    let mut registry = ToolRegistry::default();
    let name = ToolName::new("test.echo").expect("valid tool name");

    registry
        .register(EchoTool::new("test.echo"))
        .expect("register echo tool");
    let tool = registry.get(&name).expect("registered tool");
    let output = tool
        .execute(ToolInput {
            value: json!({ "message": "hello" }),
        })
        .expect("execute echo tool");

    assert_eq!(output.value, json!({ "message": "hello" }));
}

#[test]
fn registry_rejects_duplicate_names() {
    let mut registry = ToolRegistry::default();
    let name = ToolName::new("test.echo").expect("valid tool name");

    registry
        .register(EchoTool::new("test.echo"))
        .expect("register first tool");
    let error = registry
        .register(EchoTool::new("test.echo"))
        .expect_err("duplicate rejected");

    assert!(matches!(error, ToolRegistryError::DuplicateTool(duplicate) if duplicate == name));
}

#[test]
fn registry_returns_none_for_unknown_tool() {
    let registry = ToolRegistry::default();
    let name = ToolName::new("test.missing").expect("valid tool name");

    assert!(registry.get(&name).is_none());
}

#[test]
fn tool_definition_serializes_stable_shape() {
    let definition = EchoTool::new("test.echo").definition();
    let json = serde_json::to_value(definition).expect("serialize definition");

    assert_eq!(
        json,
        json!({
            "name": "test.echo",
            "description": "echo input",
            "input_schema": {
                "type": "object",
                "additionalProperties": true
            }
        })
    );
}
