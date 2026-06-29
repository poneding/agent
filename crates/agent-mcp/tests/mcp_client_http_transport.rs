use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use agent_mcp::{
    FixtureMcpClientHttpTransport, McpClientHttpRequest, McpClientHttpReqwestTransport,
    McpClientHttpResponse, McpClientHttpTransport as _, McpEnvValue, McpServerConfig, McpServerId,
    McpTransportConfig,
};
use serde_json::{Value, json};

#[derive(Debug)]
struct RecordedRequest {
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

fn spawn_server(status: &'static str, body: &'static str) -> (String, Receiver<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind server");
    let addr = listener.local_addr().expect("server addr");
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let recorded = read_request(stream, status, body);
        tx.send(recorded).expect("send recorded request");
    });

    (format!("http://{addr}/mcp"), rx)
}

fn read_request(mut stream: TcpStream, status: &str, response_body: &str) -> RecordedRequest {
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut request_line = String::new();
    reader.read_line(&mut request_line).expect("request line");
    let path = request_line
        .split_whitespace()
        .nth(1)
        .expect("path")
        .to_string();

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("header line");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        let Some((name, value)) = line.trim_end().split_once(':') else {
            continue;
        };
        let name = name.to_ascii_lowercase();
        let value = value.trim().to_string();
        if name == "content-length" {
            content_length = value.parse().expect("content length");
        }
        headers.insert(name, value);
    }

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).expect("body");
    let body = String::from_utf8(body).expect("utf8 body");

    let response = format!(
        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response_body}",
        response_body.len()
    );
    stream.write_all(response.as_bytes()).expect("response");
    stream.flush().expect("flush");

    RecordedRequest {
        path,
        headers,
        body,
    }
}

fn server_id() -> McpServerId {
    McpServerId::new("http-server").expect("server id")
}

fn streamable_config(url: String, headers: BTreeMap<String, McpEnvValue>) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Server",
        McpTransportConfig::StreamableHttp { url, headers },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

#[tokio::test]
async fn mcp_client_http_transport_fixture_records_and_returns_response() {
    let response = McpClientHttpResponse::new(200, json!({"ok": true})).expect("response");
    let mut transport = FixtureMcpClientHttpTransport::from_responses([Ok(response.clone())]);
    let request = McpClientHttpRequest::new(
        "http://localhost/mcp",
        BTreeMap::new(),
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        1_000,
    )
    .expect("request");

    let actual = transport.send_request(request.clone()).await.expect("send");

    assert_eq!(actual, response);
    assert_eq!(transport.sent_requests(), &[request]);
}

#[test]
fn mcp_client_http_transport_request_from_config_resolves_literal_headers() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "x-test".to_string(),
        McpEnvValue::Literal("fixture".to_string()),
    );
    let config = streamable_config("http://localhost/mcp".to_string(), headers);

    let request = McpClientHttpRequest::from_config(&config, json!({"ok": true}))
        .expect("request from config");

    assert_eq!(request.url, "http://localhost/mcp");
    assert_eq!(request.headers["x-test"], "fixture");
    assert_eq!(request.timeout_ms, 1_000);
    assert_eq!(request.body_json["ok"], true);
}

#[test]
fn mcp_client_http_transport_rejects_invalid_configs_before_request() {
    let disabled =
        streamable_config("http://localhost/mcp".to_string(), BTreeMap::new()).disabled();
    let disabled_error =
        McpClientHttpRequest::from_config(&disabled, json!({})).expect_err("disabled");
    assert!(disabled_error.to_string().contains("disabled"));

    let non_http = McpServerConfig::new(
        server_id(),
        "Stdio",
        McpTransportConfig::stdio("mcp-server"),
    )
    .expect("config");
    let non_http_error =
        McpClientHttpRequest::from_config(&non_http, json!({})).expect_err("non-http");
    assert!(non_http_error.to_string().contains("streamable http"));

    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        McpEnvValue::SecretRef("mcp.token".to_string()),
    );
    let secret_ref = streamable_config("http://localhost/mcp".to_string(), headers);
    let secret_error =
        McpClientHttpRequest::from_config(&secret_ref, json!({})).expect_err("secret ref");
    let message = secret_error.to_string();
    assert!(message.contains("unresolved secret ref"));
    assert!(!message.contains("authorization"));
    assert!(!message.contains("mcp.token"));
}

#[tokio::test]
async fn mcp_client_http_transport_reqwest_posts_json_and_headers() {
    let (url, rx) = spawn_server("200 OK", r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#);
    let mut headers = BTreeMap::new();
    headers.insert("x-test".to_string(), "fixture".to_string());
    let request = McpClientHttpRequest::new(
        url,
        headers,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        5_000,
    )
    .expect("request");
    let mut transport = McpClientHttpReqwestTransport::new();

    let response = transport.send_request(request).await.expect("send");
    let recorded = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recorded request");
    let body = serde_json::from_str::<Value>(&recorded.body).expect("json body");

    assert_eq!(response.status, 200);
    assert_eq!(response.body_json["result"]["ok"], true);
    assert_eq!(recorded.path, "/mcp");
    assert_eq!(recorded.headers["x-test"], "fixture");
    assert_eq!(body["method"], "initialize");
}

#[tokio::test]
async fn mcp_client_http_transport_reqwest_non_success_status_is_redacted() {
    let (url, rx) = spawn_server(
        "503 Service Unavailable",
        r#"{"error":"sk-secret-token api_key password"}"#,
    );
    let request = McpClientHttpRequest::new(
        url,
        BTreeMap::new(),
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        5_000,
    )
    .expect("request");
    let mut transport = McpClientHttpReqwestTransport::new();

    let error = transport.send_request(request).await.expect_err("status");
    let _recorded = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recorded request");
    let message = error.to_string();

    assert!(message.contains("HTTP status 503"));
    assert!(!message.contains("sk-secret-token"));
    assert!(!message.contains("api_key"));
    assert!(!message.contains("password"));
}
