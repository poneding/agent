use agent_mcp::{
    McpCreateMessageRequestParams, McpSamplingApprovalDecision, McpSamplingApprovalPolicy,
    McpSamplingApprovalStatus, McpSamplingContent, McpSamplingDecisionKind,
    McpSamplingIncludeContext, McpSamplingMessage, McpSamplingRole, McpSamplingTaskMetadata,
    McpSamplingToolLoopPolicy, McpServerId, build_mcp_sampling_approval_report,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("sampling-server").expect("server id")
}

fn basic_request() -> McpCreateMessageRequestParams {
    McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text(
            "summarize this fixture input",
        ))],
        128,
    )
}

#[test]
fn mcp_sampling_productization_serializes_approval_request_shape() {
    let report = build_mcp_sampling_approval_report(
        "sample-review-1",
        server_id(),
        &basic_request(),
        None,
        McpSamplingApprovalPolicy::default(),
        None,
    )
    .expect("approval report");

    assert_eq!(report.terminal_status, McpSamplingApprovalStatus::Pending);
    assert!(!report.approved);
    assert_eq!(report.request.review.message_count, 1);
    assert_eq!(report.request.review.text_block_count, 1);
    assert_eq!(report.request.review.max_tokens, 128);

    let value = serde_json::to_value(&report).expect("report json");
    assert_eq!(value["request"]["request_id"], "sample-review-1");
    assert_eq!(value["request"]["status"], "pending");
    assert_eq!(value["request"]["review"]["message_count"], 1);
}

#[test]
fn mcp_sampling_productization_includes_task_metadata_without_prompt_text() {
    let mut request = basic_request().with_system_prompt("system prompt with sk-secret-token");
    request.include_context = McpSamplingIncludeContext::ThisServer;
    let task = McpSamplingTaskMetadata::new("task-123")
        .expect("task")
        .with_conversation_id("conversation-123")
        .expect("conversation")
        .with_turn_id("turn-123")
        .expect("turn")
        .with_title("Summarize workspace")
        .expect("title");

    let report = build_mcp_sampling_approval_report(
        "sample-review-2",
        server_id(),
        &request,
        Some(task),
        McpSamplingApprovalPolicy::default(),
        None,
    )
    .expect("approval report");

    assert_eq!(
        report.request.task.as_ref().expect("task").task_id,
        "task-123"
    );
    assert!(report.request.review.has_system_prompt);
    assert_eq!(
        report.request.review.requested_context,
        McpSamplingIncludeContext::ThisServer
    );
    let json = serde_json::to_string(&report).expect("report json");
    assert!(!json.contains("system prompt"));
    assert!(!json.contains("sk-secret-token"));
}

#[test]
fn mcp_sampling_productization_default_denies_tool_enabled_sampling_loops() {
    let mut request = basic_request();
    request.tools = vec![json!({"name": "local.shell"})];
    request.tool_choice = Some(json!({"type": "auto"}));

    let report = build_mcp_sampling_approval_report(
        "sample-review-tools",
        server_id(),
        &request,
        None,
        McpSamplingApprovalPolicy::default(),
        None,
    )
    .expect("approval report");

    assert!(report.request.review.requested_tools);
    assert!(report.request.review.tool_choice_requested);
    assert!(report.request.review.tool_loop_default_denied);
    assert!(!report.request.review.validation.accepted);
    assert!(
        report
            .request
            .review
            .validation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsupported_sampling_tools")
    );
    let json = serde_json::to_string(&report).expect("report json");
    assert!(!json.contains("local.shell"));
}

#[test]
fn mcp_sampling_productization_can_model_explicit_tool_loop_policy_without_default_allow() {
    let mut request = basic_request();
    request.tools = vec![json!({"name": "local.read_only"})];
    let policy = McpSamplingApprovalPolicy {
        require_human_review: true,
        tool_loop_policy: McpSamplingToolLoopPolicy::AllowIfExplicitlyApproved,
    };

    let report = build_mcp_sampling_approval_report(
        "sample-review-explicit-tools",
        server_id(),
        &request,
        None,
        policy,
        None,
    )
    .expect("approval report");

    assert!(report.request.review.requested_tools);
    assert!(!report.request.review.tool_loop_default_denied);
    assert!(report.request.review.validation.accepted);
    assert_eq!(
        McpSamplingApprovalPolicy::default().tool_loop_policy,
        McpSamplingToolLoopPolicy::Deny
    );
}

#[test]
fn mcp_sampling_productization_records_decisions_and_redacts_reasons() {
    let request = basic_request();
    let decision = McpSamplingApprovalDecision::new(
        "sample-review-3",
        McpSamplingDecisionKind::Deny,
        "policy rejected unsafe sampling",
        "desktop",
    )
    .expect("decision");

    let report = build_mcp_sampling_approval_report(
        "sample-review-3",
        server_id(),
        &request,
        None,
        McpSamplingApprovalPolicy::default(),
        Some(decision),
    )
    .expect("approval report");

    assert_eq!(report.terminal_status, McpSamplingApprovalStatus::Denied);
    assert!(!report.approved);
    assert_eq!(
        report.decision.as_ref().expect("decision").decision,
        McpSamplingDecisionKind::Deny
    );

    let secret_decision = McpSamplingApprovalDecision::new(
        "sample-review-4",
        McpSamplingDecisionKind::Deny,
        "api_key sk-secret-token",
        "desktop",
    )
    .expect_err("secret reason rejected");
    assert!(!secret_decision.to_string().contains("sk-secret-token"));
}

#[test]
fn mcp_sampling_productization_counts_tool_result_content_without_exposing_payloads() {
    let tool_result = McpSamplingContent::tool_result(
        "tool-1",
        vec![McpSamplingContent::text(
            "secret tool output sk-secret-token",
        )],
    )
    .expect("tool result");
    let request = McpCreateMessageRequestParams::new(
        vec![
            McpSamplingMessage::assistant(
                McpSamplingContent::tool_use("tool-1", "read_only", json!({"path": "README.md"}))
                    .expect("tool use"),
            ),
            McpSamplingMessage::new(McpSamplingRole::User, tool_result),
        ],
        128,
    );

    let report = build_mcp_sampling_approval_report(
        "sample-review-tool-result",
        server_id(),
        &request,
        None,
        McpSamplingApprovalPolicy {
            require_human_review: true,
            tool_loop_policy: McpSamplingToolLoopPolicy::AllowIfExplicitlyApproved,
        },
        None,
    )
    .expect("approval report");

    assert_eq!(report.request.review.tool_use_block_count, 1);
    assert_eq!(report.request.review.tool_result_block_count, 1);
    assert_eq!(report.request.review.text_block_count, 1);
    let json = serde_json::to_string(&report).expect("report json");
    assert!(!json.contains("secret tool output"));
    assert!(!json.contains("sk-secret-token"));
    assert!(!json.contains("README.md"));
}
