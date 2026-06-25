use std::sync::Arc;

use agent_core::{RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopStatus};
use agent_plugin::{
    PluginProcessError, PluginToolHost, PluginToolInvocation, parse_plugin_manifest_json,
    register_plugin_tools,
};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_tool::{ToolOutput, ToolRegistry};
use agent_types::{
    ConversationId, CorrelationId, RuntimeEventPayload, TurnId, TurnState, WorkspaceId,
};
use serde_json::json;

const TEMPLATE: &str = include_str!("../../../docs/templates/plugin-manifest.template.json");

#[test]
fn m11_plugin_runtime_integration_tool_runs_through_policy_and_runtime_events() {
    let manifest = parse_plugin_manifest_json(TEMPLATE).expect("parse manifest");
    let mut registry = ToolRegistry::default();
    register_plugin_tools(&mut registry, &manifest, Arc::new(StaticPluginHost))
        .expect("register plugin tool");
    let policy = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "example.read").expect("scope"),
        PolicyDecision::Allow {
            reason: "plugin tool allowed".to_string(),
        },
    );

    let outcome = ToolLoop
        .run(
            runtime_ids(),
            ToolCallRequest {
                call_id: "plugin-call-1".to_string(),
                tool_name: "example.read".to_string(),
                input_json: json!({ "path": "README.md" }),
            },
            &registry,
            &policy,
        )
        .expect("tool loop");

    assert_eq!(outcome.status, ToolLoopStatus::Completed);
    assert_eq!(outcome.turn_state, TurnState::Observed);
    assert_eq!(outcome.observation.as_deref(), Some("plugin output"));
    let names: Vec<&str> = outcome
        .events
        .iter()
        .map(|event| match &event.payload {
            RuntimeEventPayload::ToolCallProposed(_) => "proposed",
            RuntimeEventPayload::PermissionDecisionRecorded(_) => "decision",
            RuntimeEventPayload::ToolExecutionStarted(_) => "started",
            RuntimeEventPayload::ToolExecutionCompleted(_) => "completed",
            RuntimeEventPayload::ToolObservationRecorded(_) => "observed",
            _ => "unexpected",
        })
        .collect();
    assert_eq!(
        names,
        vec!["proposed", "decision", "started", "completed", "observed"]
    );
}

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

#[derive(Debug)]
struct StaticPluginHost;

impl PluginToolHost for StaticPluginHost {
    fn invoke(&self, request: PluginToolInvocation) -> Result<ToolOutput, PluginProcessError> {
        assert_eq!(request.plugin_name.as_str(), "example-plugin");
        assert_eq!(request.tool_name.as_str(), "example.read");
        assert_eq!(request.input, json!({ "path": "README.md" }));
        Ok(ToolOutput {
            value: json!({ "content": "plugin output" }),
        })
    }
}
