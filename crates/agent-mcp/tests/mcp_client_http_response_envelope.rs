use std::collections::BTreeMap;

use agent_mcp::{
    McpClientHttpResponse, McpClientHttpResponseEnvelope, McpClientHttpResponseFrame,
    McpClientHttpResponseMode, McpClientHttpSessionMetadata, McpClientHttpSseResponse,
    McpJsonRpcId, find_mcp_client_http_response_frame, parse_mcp_client_http_session_metadata,
    parse_mcp_client_http_sse_events,
};
use serde_json::json;

#[test]
fn mcp_client_http_response_envelope_wraps_json_mode_with_metadata() {
    let mode = McpClientHttpResponseMode::Json(
        McpClientHttpResponse::new(
            200,
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "result": {
                    "ok": true
                }
            }),
        )
        .expect("response"),
    );
    let metadata = McpClientHttpSessionMetadata {
        session_id: Some("session-7".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: None,
    };

    let envelope = McpClientHttpResponseEnvelope::new(mode, metadata.clone());

    assert_eq!(envelope.metadata, metadata);
    let frame = find_mcp_client_http_response_frame(&envelope.mode, &McpJsonRpcId::Number(7))
        .expect("frame");
    match frame {
        McpClientHttpResponseFrame::Success(success) => {
            assert_eq!(success.result["ok"], true);
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_response_envelope_keeps_existing_json_fixture_conversion() {
    let response = McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {}
        }),
    )
    .expect("response");

    let envelope: McpClientHttpResponseEnvelope = response.into();

    assert_eq!(envelope.metadata, McpClientHttpSessionMetadata::default());
    assert!(matches!(envelope.mode, McpClientHttpResponseMode::Json(_)));
}

#[test]
fn mcp_client_http_response_envelope_supports_sse_metadata_extraction() {
    let events = parse_mcp_client_http_sse_events(
        "id: event-9\ndata: {\"jsonrpc\":\"2.0\",\"id\":9,\"result\":{\"ok\":true}}\n\n",
    )
    .expect("events");
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));
    let mut headers = BTreeMap::new();
    headers.insert("mcp-session-id".to_string(), "session-9".to_string());

    let metadata = parse_mcp_client_http_session_metadata(&headers, &mode).expect("metadata");
    let envelope = McpClientHttpResponseEnvelope::new(mode, metadata);

    assert_eq!(envelope.metadata.session_id.as_deref(), Some("session-9"));
    assert_eq!(envelope.metadata.last_event_id.as_deref(), Some("event-9"));
    let frame = find_mcp_client_http_response_frame(&envelope.mode, &McpJsonRpcId::Number(9))
        .expect("frame");
    assert!(matches!(frame, McpClientHttpResponseFrame::Success(_)));
}
