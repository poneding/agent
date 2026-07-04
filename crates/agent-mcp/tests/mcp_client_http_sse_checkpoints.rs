use std::collections::BTreeMap;

use agent_mcp::{
    FixtureMcpClientHttpTransport, McpClient as _, McpClientHttpResponse,
    McpClientHttpResponseEnvelope, McpClientHttpResponseMode, McpClientHttpSessionMetadata,
    McpClientHttpSseResponse, McpHttpClient, McpServerConfig, McpServerId, McpTransportConfig,
    parse_mcp_client_http_session_metadata, parse_mcp_client_http_sse_events,
};
use serde_json::{Value, json};

fn server_id() -> McpServerId {
    McpServerId::new("http-sse-checkpoint").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP SSE Checkpoint",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost/mcp".to_string(),
            headers: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn sse_envelope(id: i64, event_id: &str, result: Value) -> McpClientHttpResponseEnvelope {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    let body = format!("id: {event_id}\nevent: message\ndata: {frame}\n\n");
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(
        parse_mcp_client_http_sse_events(&body).expect("events"),
    ));
    let metadata =
        parse_mcp_client_http_session_metadata(&BTreeMap::new(), &mode).expect("metadata");
    McpClientHttpResponseEnvelope::new(mode, metadata)
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2025-11-25",
        "serverInfo": {
            "name": "http-sse-checkpoint-server",
            "version": "1.0.0"
        },
        "capabilities": {
            "tools": {}
        }
    })
}

fn tools_result() -> Value {
    json!({
        "tools": []
    })
}

#[tokio::test]
async fn mcp_client_http_sse_checkpoints_update_after_accepted_response() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(sse_envelope(1, "init-event", initialize_result())),
        Ok(sse_envelope(2, "tools-event", tools_result())),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    assert_eq!(
        client
            .session_metadata_snapshot()
            .await
            .last_event_id
            .as_deref(),
        Some("init-event")
    );

    client.list_tools(&config.id).await.expect("tools");

    assert_eq!(
        client
            .session_metadata_snapshot()
            .await
            .last_event_id
            .as_deref(),
        Some("tools-event")
    );
}

#[tokio::test]
async fn mcp_client_http_sse_checkpoints_ignore_mismatched_response_id() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(sse_envelope(1, "init-event", initialize_result())),
        Ok(sse_envelope(99, "mismatched-event", tools_result())),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let error = client
        .list_tools(&config.id)
        .await
        .expect_err("mismatched id");

    assert!(error.to_string().contains("matching response id missing"));
    assert_eq!(
        client
            .session_metadata_snapshot()
            .await
            .last_event_id
            .as_deref(),
        Some("init-event")
    );
}

#[tokio::test]
async fn mcp_client_http_sse_checkpoints_ignore_failed_result_mapping() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(sse_envelope(1, "init-event", initialize_result())),
        Ok(sse_envelope(
            2,
            "malformed-tools-event",
            json!({
                "tools": "not-an-array"
            }),
        )),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let error = client
        .list_tools(&config.id)
        .await
        .expect_err("invalid tools result");

    assert!(
        error
            .to_string()
            .contains("invalid MCP client HTTP tools/list result")
    );
    assert_eq!(
        client
            .session_metadata_snapshot()
            .await
            .last_event_id
            .as_deref(),
        Some("init-event")
    );
}

#[tokio::test]
async fn mcp_client_http_sse_checkpoints_preserve_state_for_json_response() {
    let config = config();
    let json_tools = McpClientHttpResponseEnvelope::new(
        McpClientHttpResponse::new(
            200,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "result": {
                    "tools": []
                }
            }),
        )
        .expect("response")
        .into(),
        McpClientHttpSessionMetadata::default(),
    );
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(sse_envelope(1, "init-event", initialize_result())),
        Ok(json_tools),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    client.list_tools(&config.id).await.expect("tools");

    assert_eq!(
        client
            .session_metadata_snapshot()
            .await
            .last_event_id
            .as_deref(),
        Some("init-event")
    );
}
