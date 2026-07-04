use agent_mcp::{
    FixtureMcpClientStdioTransport, McpClientStdioResponseFrame, McpClientStdioSession,
    McpClientStdioSessionStatus, McpJsonRpcId,
};
use serde_json::{Value, json};

#[tokio::test]
async fn mcp_client_stdio_session_sends_request_and_receives_matching_success() {
    let transport = FixtureMcpClientStdioTransport::from_inbound_frames([json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "ok": true
        }
    })
    .to_string()]);
    let mut session = McpClientStdioSession::new(transport);

    let response = session
        .send_request("initialize", json!({"clientInfo": {"name": "agent"}}))
        .await
        .expect("matching response");

    match response {
        McpClientStdioResponseFrame::Success(success) => {
            assert_eq!(success.id, McpJsonRpcId::Number(1));
            assert_eq!(success.result["ok"], true);
        }
        other => panic!("expected success, got {other:?}"),
    }

    let sent = session.transport().sent_frames();
    assert_eq!(sent.len(), 1);
    let request = serde_json::from_str::<Value>(sent[0].trim_end()).expect("request frame");
    assert_eq!(request["id"], 1);
    assert_eq!(request["method"], "initialize");

    let summary = session.summary();
    assert_eq!(summary.status, McpClientStdioSessionStatus::Created);
    assert_eq!(summary.requests_sent, 1);
    assert_eq!(summary.responses_received, 1);
    assert_eq!(summary.error_frames, 0);
}

#[tokio::test]
async fn mcp_client_stdio_session_rejects_mismatched_response_id() {
    let transport = FixtureMcpClientStdioTransport::from_inbound_frames([json!({
        "jsonrpc": "2.0",
        "id": 99,
        "result": {}
    })
    .to_string()]);
    let mut session = McpClientStdioSession::new(transport);

    let error = session
        .send_request("tools/list", json!({}))
        .await
        .expect_err("mismatched id");

    assert!(error.to_string().contains("unexpected response id"));
    let summary = session.summary();
    assert_eq!(summary.status, McpClientStdioSessionStatus::Failed);
    assert_eq!(summary.requests_sent, 1);
    assert_eq!(summary.responses_received, 1);
    assert_eq!(summary.error_frames, 1);
}

#[tokio::test]
async fn mcp_client_stdio_session_rejects_eof_while_waiting() {
    let transport = FixtureMcpClientStdioTransport::default();
    let mut session = McpClientStdioSession::new(transport);

    let error = session
        .send_request("tools/list", json!({}))
        .await
        .expect_err("eof");

    assert!(error.to_string().contains("closed before response"));
    let summary = session.summary();
    assert_eq!(summary.status, McpClientStdioSessionStatus::Failed);
    assert_eq!(summary.requests_sent, 1);
    assert_eq!(summary.responses_received, 0);
    assert_eq!(summary.error_frames, 1);
}

#[tokio::test]
async fn mcp_client_stdio_session_summary_counts_and_closes_idempotently() {
    let transport = FixtureMcpClientStdioTransport::from_inbound_frames([json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {}
    })
    .to_string()]);
    let mut session = McpClientStdioSession::new(transport);

    session
        .send_request("initialize", json!({}))
        .await
        .expect("response");
    let first = session.close("done");
    let second = session.close("later");

    assert_eq!(first, second);
    assert_eq!(second.status, McpClientStdioSessionStatus::Closed);
    assert_eq!(second.requests_sent, 1);
    assert_eq!(second.responses_received, 1);
    assert_eq!(second.error_frames, 0);
    assert_eq!(second.close_reason.as_deref(), Some("done"));
}
