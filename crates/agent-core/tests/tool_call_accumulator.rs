use agent_core::{AccumulatedToolCall, ToolCallAccumulator};
use serde_json::json;

#[test]
fn tool_call_accumulator_accepts_valid_complete_call() {
    let result = ToolCallAccumulator::default().push_tool_call(
        "call-1",
        "workspace.read_file",
        json!({ "path": "README.md" }),
    );

    match result {
        AccumulatedToolCall::Ready(call) => {
            assert_eq!(call.call_id, "call-1");
            assert_eq!(call.tool_name, "workspace.read_file");
            assert_eq!(call.input_json, json!({ "path": "README.md" }));
        }
        AccumulatedToolCall::Observation(observation) => {
            panic!("expected ready call, got observation: {observation:?}");
        }
    }
}

#[test]
fn tool_call_accumulator_reports_duplicate_call_id() {
    let mut accumulator = ToolCallAccumulator::default();

    let _ = accumulator.push_tool_call("call-1", "workspace.read_file", json!({}));
    let duplicate = accumulator.push_tool_call("call-1", "workspace.read_file", json!({}));

    match duplicate {
        AccumulatedToolCall::Ready(call) => {
            panic!("expected duplicate observation, got ready call: {call:?}");
        }
        AccumulatedToolCall::Observation(observation) => {
            assert_eq!(observation.call_id, "call-1");
            assert_eq!(observation.tool_name, "workspace.read_file");
            assert!(observation.observation.contains("duplicate tool call id"));
            assert!(!observation.truncated);
        }
    }
}

#[test]
fn tool_call_accumulator_reports_invalid_tool_name() {
    let result =
        ToolCallAccumulator::default().push_tool_call("call-1", "Workspace.ReadFile", json!({}));

    match result {
        AccumulatedToolCall::Ready(call) => {
            panic!("expected invalid-name observation, got ready call: {call:?}");
        }
        AccumulatedToolCall::Observation(observation) => {
            assert_eq!(observation.call_id, "call-1");
            assert_eq!(observation.tool_name, "Workspace.ReadFile");
            assert!(observation.observation.contains("invalid tool name"));
        }
    }
}

#[test]
fn tool_call_accumulator_reports_non_object_input() {
    let result = ToolCallAccumulator::default().push_tool_call(
        "call-1",
        "workspace.read_file",
        json!("README.md"),
    );

    match result {
        AccumulatedToolCall::Ready(call) => {
            panic!("expected invalid-input observation, got ready call: {call:?}");
        }
        AccumulatedToolCall::Observation(observation) => {
            assert_eq!(observation.call_id, "call-1");
            assert_eq!(observation.tool_name, "workspace.read_file");
            assert!(
                observation
                    .observation
                    .contains("tool input must be a JSON object")
            );
        }
    }
}
