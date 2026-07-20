use std::sync::{Arc, Mutex};

use ruse_plugin::{
    PluginHookEvent, PluginHookOutcome, PluginHookRunner, PluginManifest, PluginProcessError,
    PluginTool, PluginToolHost, PluginToolInvocation, parse_plugin_manifest_json,
    register_plugin_tools,
};
use ruse_tool::{Tool, ToolInput, ToolOutput, ToolRegistry};
use serde_json::{Value, json};

const TEMPLATE: &str = include_str!("../../../docs/templates/plugin-manifest.template.json");

#[test]
fn plugin_tools_hooks_tool_definition_matches_manifest() {
    let manifest = template_manifest();
    let host = Arc::new(RecordingHost::success(json!({ "content": "hello" })));

    let tool = PluginTool::from_manifest(&manifest, &manifest.tools[0], host)
        .expect("plugin tool from manifest");

    let definition = tool.definition();
    assert_eq!(definition.name, manifest.tools[0].name);
    assert_eq!(definition.description, manifest.tools[0].description);
    assert_eq!(definition.input_schema, manifest.tools[0].input_schema);
}

#[test]
fn plugin_tools_hooks_tool_registers_in_tool_registry() {
    let manifest = template_manifest();
    let host = Arc::new(RecordingHost::success(json!({ "content": "hello" })));
    let mut registry = ToolRegistry::default();

    let registered =
        register_plugin_tools(&mut registry, &manifest, host).expect("register plugin tools");

    assert_eq!(registered, vec![manifest.tools[0].name.clone()]);
    assert!(registry.get(&manifest.tools[0].name).is_some());
}

#[test]
fn plugin_tools_hooks_tool_rejects_duplicate_registration() {
    let manifest = template_manifest();
    let host = Arc::new(RecordingHost::success(json!({ "content": "hello" })));
    let mut registry = ToolRegistry::default();
    register_plugin_tools(&mut registry, &manifest, host.clone()).expect("first registration");

    let err = register_plugin_tools(&mut registry, &manifest, host).unwrap_err();

    assert!(err.to_string().contains("duplicate tool"));
}

#[test]
fn plugin_tools_hooks_tool_invokes_host_and_returns_output() {
    let manifest = template_manifest();
    let host = Arc::new(RecordingHost::success(json!({ "content": "from host" })));
    let tool = PluginTool::from_manifest(&manifest, &manifest.tools[0], host.clone())
        .expect("plugin tool from manifest");

    let output = tool
        .execute(ToolInput {
            value: json!({ "path": "README.md" }),
        })
        .expect("execute plugin tool");

    assert_eq!(output.value, json!({ "content": "from host" }));
    let requests = host.requests.lock().expect("requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].plugin_name.as_str(), "example-plugin");
    assert_eq!(requests[0].tool_name.as_str(), "example.read");
    assert_eq!(requests[0].input, json!({ "path": "README.md" }));
    assert!(!requests[0].call_id.trim().is_empty());
}

#[test]
fn plugin_tools_hooks_tool_maps_host_error_to_tool_error() {
    let manifest = template_manifest();
    let host = Arc::new(RecordingHost::failure(PluginProcessError::Plugin {
        error_code: "plugin_error".to_string(),
        message: "host failed".to_string(),
        recoverable: true,
    }));
    let tool = PluginTool::from_manifest(&manifest, &manifest.tools[0], host).expect("plugin tool");

    let err = tool
        .execute(ToolInput {
            value: json!({ "path": "README.md" }),
        })
        .unwrap_err();

    assert!(err.to_string().contains("host failed"));
}

#[test]
fn plugin_tools_hooks_hook_runner_records_success_and_failure() {
    let mut runner = PluginHookRunner::default();
    runner.register("audit", |event| {
        assert_eq!(event, PluginHookEvent::TurnStarted);
        Ok(PluginHookOutcome {
            hook_name: "audit".to_string(),
            emitted_messages: vec!["started".to_string()],
        })
    });
    runner.register("failing", |_event| Err("hook failed".to_string()));

    let outcomes = runner.run(PluginHookEvent::TurnStarted, "corr-1");
    let repeated = runner.run(PluginHookEvent::TurnStarted, "corr-1");

    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].hook_name, "audit");
    assert_eq!(outcomes[0].emitted_messages, vec!["started"]);
    assert_eq!(outcomes[1].hook_name, "failing");
    assert_eq!(outcomes[1].emitted_messages, vec!["hook failed"]);
    assert!(repeated.is_empty());
}

fn template_manifest() -> PluginManifest {
    parse_plugin_manifest_json(TEMPLATE).expect("parse manifest")
}

#[derive(Debug)]
struct RecordingHost {
    result: Result<Value, PluginProcessError>,
    requests: Mutex<Vec<PluginToolInvocation>>,
}

impl RecordingHost {
    fn success(output: Value) -> Self {
        Self {
            result: Ok(output),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn failure(error: PluginProcessError) -> Self {
        Self {
            result: Err(error),
            requests: Mutex::new(Vec::new()),
        }
    }
}

impl PluginToolHost for RecordingHost {
    fn invoke(&self, request: PluginToolInvocation) -> Result<ToolOutput, PluginProcessError> {
        self.requests
            .lock()
            .expect("requests")
            .push(request.clone());
        self.result.clone().map(|value| ToolOutput { value })
    }
}
