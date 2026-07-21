use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ruse_mcp::{
    McpClient as _, McpClientHttpRequest, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpSessionMetadata, McpClientHttpTransport, McpEnvValue, McpError, McpHttpClient,
    McpServerConfig, McpServerId, McpTransportConfig,
};
use serde_json::json;

struct RecordingTransport {
    inbound_responses: VecDeque<Result<McpClientHttpResponseEnvelope, McpError>>,
    sent_requests: Arc<Mutex<Vec<McpClientHttpRequest>>>,
}

impl RecordingTransport {
    fn from_responses<I, R>(responses: I) -> (Self, Arc<Mutex<Vec<McpClientHttpRequest>>>)
    where
        I: IntoIterator<Item = Result<R, McpError>>,
        R: Into<McpClientHttpResponseEnvelope>,
    {
        let sent_requests = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                inbound_responses: responses
                    .into_iter()
                    .map(|response| response.map(Into::into))
                    .collect(),
                sent_requests: Arc::clone(&sent_requests),
            },
            sent_requests,
        )
    }
}

#[async_trait]
impl McpClientHttpTransport for RecordingTransport {
    async fn send_request(
        &mut self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponseEnvelope, McpError> {
        self.sent_requests
            .lock()
            .expect("sent requests")
            .push(request);
        self.inbound_responses
            .pop_front()
            .unwrap_or_else(|| Err(McpError::Client("missing response".to_string())))
    }
}

fn server_id() -> McpServerId {
    McpServerId::new("http-session-headers").expect("server id")
}

fn config_with_headers(headers: BTreeMap<String, McpEnvValue>) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Session Headers",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost/mcp".to_string(),
            headers,
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn config() -> McpServerConfig {
    config_with_headers(BTreeMap::new())
}

fn initialize_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-11-25",
                "serverInfo": {
                    "name": "http-session-server",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "tools": {}
                }
            }
        }),
    )
    .expect("response")
}

fn tools_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": []
            }
        }),
    )
    .expect("response")
}

fn initialize_envelope_with_metadata(id: i64) -> McpClientHttpResponseEnvelope {
    McpClientHttpResponseEnvelope::new(
        initialize_response(id).into(),
        McpClientHttpSessionMetadata {
            session_id: Some("session-abc".to_string()),
            protocol_version: Some("2025-11-25".to_string()),
            last_event_id: None,
        },
    )
}

#[tokio::test]
async fn mcp_client_http_session_headers_sent_after_successful_initialize() {
    let config = config();
    let (transport, sent_requests) = RecordingTransport::from_responses([
        Ok(initialize_envelope_with_metadata(1)),
        Ok(tools_response(2).into()),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    client.list_tools(&config.id).await.expect("tools");

    let requests = sent_requests.lock().expect("sent requests");
    assert_eq!(requests.len(), 2);
    assert!(!requests[0].headers.contains_key("Mcp-Session-Id"));
    assert_eq!(
        requests[1]
            .headers
            .get("Mcp-Session-Id")
            .map(String::as_str),
        Some("session-abc")
    );
    assert_eq!(
        requests[1]
            .headers
            .get("MCP-Protocol-Version")
            .map(String::as_str),
        Some("2025-11-25")
    );
}

#[tokio::test]
async fn mcp_client_http_session_headers_not_recorded_after_invalid_initialize() {
    let config = config();
    let invalid_initialize = McpClientHttpResponseEnvelope::new(
        McpClientHttpResponse::new(
            200,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "protocolVersion": "2025-11-25",
                    "serverInfo": {
                        "version": "1.0.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }
            }),
        )
        .expect("response")
        .into(),
        McpClientHttpSessionMetadata {
            session_id: Some("session-sk-secret-token".to_string()),
            protocol_version: Some("2025-11-25".to_string()),
            last_event_id: None,
        },
    );
    let (transport, sent_requests) =
        RecordingTransport::from_responses([Ok(invalid_initialize), Ok(tools_response(2).into())]);
    let client = McpHttpClient::new(transport);

    let error = client
        .initialize(&config)
        .await
        .expect_err("invalid initialize");
    let message = error.to_string();
    let list_error = client
        .list_tools(&config.id)
        .await
        .expect_err("not initialized");

    assert!(message.contains("invalid MCP client HTTP initialize result"));
    assert!(!message.contains("sk-secret-token"));
    assert!(list_error.to_string().contains("not initialized"));
    assert_eq!(sent_requests.lock().expect("sent requests").len(), 1);
}

#[tokio::test]
async fn mcp_client_http_session_headers_reject_configured_header_conflict_safely() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "mcp-session-id".to_string(),
        McpEnvValue::Literal("configured-session".to_string()),
    );
    let config = config_with_headers(headers);
    let (transport, sent_requests) = RecordingTransport::from_responses([
        Ok(initialize_envelope_with_metadata(1)),
        Ok(tools_response(2).into()),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let error = client
        .list_tools(&config.id)
        .await
        .expect_err("header conflict");
    let message = error.to_string();

    assert!(message.contains("session header conflicts"));
    assert!(!message.contains("session-abc"));
    assert!(!message.contains("configured-session"));
    assert_eq!(sent_requests.lock().expect("sent requests").len(), 1);
}
