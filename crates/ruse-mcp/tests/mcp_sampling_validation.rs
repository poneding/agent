use ruse_mcp::{
    McpCreateMessageRequestParams, McpSamplingContent, McpSamplingContentList,
    McpSamplingDiagnosticSeverity, McpSamplingIncludeContext, McpSamplingMessage,
    McpSamplingValidationPolicy, validate_mcp_sampling_request,
};
use serde_json::json;

#[test]
fn mcp_sampling_validation_accepts_basic_text_request() {
    let request = basic_request("hello from the server", 128);

    let report = validate_mcp_sampling_request(&request, &McpSamplingValidationPolicy::default());

    assert!(report.accepted);
    assert!(report.diagnostics.is_empty());
}

#[test]
fn mcp_sampling_validation_rejects_empty_messages_and_invalid_token_bounds() {
    let empty = McpCreateMessageRequestParams::new(Vec::new(), 128);
    let empty_report =
        validate_mcp_sampling_request(&empty, &McpSamplingValidationPolicy::default());
    assert_rejected_with(&empty_report, "empty_messages");

    let zero_tokens = basic_request("hello", 0);
    let zero_report =
        validate_mcp_sampling_request(&zero_tokens, &McpSamplingValidationPolicy::default());
    assert_rejected_with(&zero_report, "invalid_max_tokens");

    let policy = McpSamplingValidationPolicy {
        max_tokens_limit: 8,
        ..McpSamplingValidationPolicy::default()
    };
    let too_many = basic_request("hello", 9);
    let too_many_report = validate_mcp_sampling_request(&too_many, &policy);
    assert_rejected_with(&too_many_report, "max_tokens_exceeded");
}

#[test]
fn mcp_sampling_validation_rejects_unsupported_tools_and_context() {
    let context_request =
        basic_request("hello", 128).with_include_context(McpSamplingIncludeContext::ThisServer);
    let context_report =
        validate_mcp_sampling_request(&context_request, &McpSamplingValidationPolicy::default());
    assert_rejected_with(&context_report, "unsupported_context");

    let mut tools_request = basic_request("hello", 128);
    tools_request.tools.push(json!({
        "name": "get_weather",
        "inputSchema": {
            "type": "object"
        }
    }));
    let tools_report =
        validate_mcp_sampling_request(&tools_request, &McpSamplingValidationPolicy::default());
    assert_rejected_with(&tools_report, "unsupported_sampling_tools");

    let mut tool_choice_request = basic_request("hello", 128);
    tool_choice_request.tool_choice = Some(json!({
        "mode": "auto"
    }));
    let tool_choice_report = validate_mcp_sampling_request(
        &tool_choice_request,
        &McpSamplingValidationPolicy::default(),
    );
    assert_rejected_with(&tool_choice_report, "unsupported_sampling_tools");
}

#[test]
fn mcp_sampling_validation_rejects_mixed_tool_result_content() {
    let tool_result =
        McpSamplingContent::tool_result("call_1", vec![McpSamplingContent::text("weather result")])
            .expect("tool result");
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContentList::from_many(
            [McpSamplingContent::text("Here is the result"), tool_result],
        ))],
        128,
    );

    let report = validate_mcp_sampling_request(&request, &McpSamplingValidationPolicy::default());

    assert_rejected_with(&report, "mixed_tool_result_content");
}

#[test]
fn mcp_sampling_validation_rejects_unbalanced_tool_use_results() {
    let first_tool =
        McpSamplingContent::tool_use("call_1", "get_weather", json!({})).expect("first tool");
    let second_tool =
        McpSamplingContent::tool_use("call_2", "get_weather", json!({})).expect("second tool");
    let one_result =
        McpSamplingContent::tool_result("call_1", vec![McpSamplingContent::text("weather result")])
            .expect("tool result");
    let request = McpCreateMessageRequestParams::new(
        vec![
            McpSamplingMessage::assistant(McpSamplingContentList::from_many([
                first_tool,
                second_tool,
            ])),
            McpSamplingMessage::user(one_result),
            McpSamplingMessage::assistant(McpSamplingContent::text("final answer")),
        ],
        128,
    );

    let report = validate_mcp_sampling_request(&request, &McpSamplingValidationPolicy::default());

    assert_rejected_with(&report, "unbalanced_tool_results");
}

#[test]
fn mcp_sampling_validation_redacts_prompt_content_from_diagnostics() {
    let secret_prompt = "sk-secret-token appears in prompt text";
    let policy = McpSamplingValidationPolicy {
        max_text_chars: 8,
        ..McpSamplingValidationPolicy::default()
    };
    let request = basic_request(secret_prompt, 128).with_system_prompt(secret_prompt);

    let report = validate_mcp_sampling_request(&request, &policy);

    assert_rejected_with(&report, "text_too_large");
    let rendered = serde_json::to_string(&report).expect("json");
    assert!(!rendered.contains(secret_prompt));
    assert!(!rendered.contains("sk-secret-token"));
}

fn basic_request(text: &str, max_tokens: u32) -> McpCreateMessageRequestParams {
    McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text(text))],
        max_tokens,
    )
}

fn assert_rejected_with(report: &ruse_mcp::McpSamplingValidationReport, code: &str) {
    assert!(!report.accepted, "report should reject request: {report:?}");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code
                && diagnostic.severity == McpSamplingDiagnosticSeverity::Error),
        "missing diagnostic code {code}: {:?}",
        report.diagnostics
    );
}
