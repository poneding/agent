use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use ruse_mcp::{
    FixtureMcpClientHttpStreamTransport, McpClientHttpReqwestTransport,
    McpClientHttpSessionMetadata, McpClientHttpStreamOpen, McpClientHttpStreamRequest,
    McpClientHttpStreamTransport as _, parse_mcp_client_http_sse_events,
};

#[derive(Debug)]
struct RecordedRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

fn spawn_server(
    status: &'static str,
    content_type: &'static str,
    body: &'static str,
) -> (String, Receiver<RecordedRequest>) {
    spawn_server_with_headers(status, content_type, body, Vec::new())
}

fn spawn_server_with_headers(
    status: &'static str,
    content_type: &'static str,
    body: &'static str,
    extra_headers: Vec<(&'static str, &'static str)>,
) -> (String, Receiver<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind server");
    let addr = listener.local_addr().expect("server addr");
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let recorded = read_request(stream, status, content_type, body, &extra_headers);
        tx.send(recorded).expect("send recorded request");
    });

    (format!("http://{addr}/mcp"), rx)
}

fn read_request(
    mut stream: TcpStream,
    status: &str,
    content_type: &str,
    response_body: &str,
    extra_headers: &[(&str, &str)],
) -> RecordedRequest {
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut request_line = String::new();
    reader.read_line(&mut request_line).expect("request line");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().expect("method").to_string();
    let path = request_parts.next().expect("path").to_string();

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

    let mut response = format!("HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\n");
    for (name, value) in extra_headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str(&format!(
        "content-length: {}\r\nconnection: close\r\n\r\n{response_body}",
        response_body.len()
    ));
    stream.write_all(response.as_bytes()).expect("response");
    stream.flush().expect("flush");

    RecordedRequest {
        method,
        path,
        headers,
        body,
    }
}

#[tokio::test]
async fn mcp_client_http_stream_transport_fixture_records_and_returns_stream() {
    let events = parse_mcp_client_http_sse_events(
        "id: event-1\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n\n",
    )
    .expect("events");
    let stream = McpClientHttpStreamOpen::new(
        McpClientHttpSessionMetadata {
            session_id: Some("session-1".to_string()),
            protocol_version: Some("2025-11-25".to_string()),
            last_event_id: Some("event-1".to_string()),
        },
        events.clone(),
    );
    let mut transport = FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream.clone())]);
    let request = McpClientHttpStreamRequest::new("http://localhost/mcp", BTreeMap::new(), 1_000)
        .expect("request");

    let actual = transport.open_stream(request.clone()).await.expect("open");

    assert_eq!(actual, stream);
    assert_eq!(actual.events, events);
    assert_eq!(transport.sent_requests(), &[request]);
}

#[tokio::test]
async fn mcp_client_http_stream_transport_reqwest_sends_get_and_headers() {
    let (url, rx) = spawn_server_with_headers(
        "200 OK",
        "text/event-stream; charset=utf-8",
        "id: event-1\nevent: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n\n",
        vec![
            ("Mcp-Session-Id", "session-1"),
            ("MCP-Protocol-Version", "2025-11-25"),
        ],
    );
    let mut headers = BTreeMap::new();
    headers.insert("x-test".to_string(), "fixture".to_string());
    let request = McpClientHttpStreamRequest::new(url, headers, 5_000).expect("request");
    let mut transport = McpClientHttpReqwestTransport::new();

    let stream = transport.open_stream(request).await.expect("open");
    let recorded = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recorded request");

    assert_eq!(recorded.method, "GET");
    assert_eq!(recorded.path, "/mcp");
    assert_eq!(recorded.headers["x-test"], "fixture");
    assert_eq!(recorded.body, "");
    assert_eq!(stream.events.len(), 1);
    assert_eq!(stream.events[0].id.as_deref(), Some("event-1"));
    assert_eq!(stream.events[0].event.as_deref(), Some("message"));
    assert_eq!(stream.metadata.session_id.as_deref(), Some("session-1"));
    assert_eq!(
        stream.metadata.protocol_version.as_deref(),
        Some("2025-11-25")
    );
    assert_eq!(stream.metadata.last_event_id.as_deref(), Some("event-1"));
}

#[tokio::test]
async fn mcp_client_http_stream_transport_reqwest_rejects_non_sse_content_type_safely() {
    let (url, rx) = spawn_server("200 OK", "text/plain", "sk-secret-token should not leak");
    let request = McpClientHttpStreamRequest::new(url, BTreeMap::new(), 5_000).expect("request");
    let mut transport = McpClientHttpReqwestTransport::new();

    let error = transport
        .open_stream(request)
        .await
        .expect_err("content type");
    let _recorded = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recorded request");
    let message = error.to_string();

    assert!(message.contains("unsupported MCP client HTTP stream content type"));
    assert!(!message.contains("sk-secret-token"));
    assert!(!message.contains("text/plain"));
}

#[tokio::test]
async fn mcp_client_http_stream_transport_reqwest_non_success_status_is_redacted() {
    let (url, rx) = spawn_server(
        "503 Service Unavailable",
        "text/event-stream",
        "data: sk-secret-token api_key password\n\n",
    );
    let request = McpClientHttpStreamRequest::new(url, BTreeMap::new(), 5_000).expect("request");
    let mut transport = McpClientHttpReqwestTransport::new();

    let error = transport.open_stream(request).await.expect_err("status");
    let _recorded = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recorded request");
    let message = error.to_string();

    assert!(message.contains("HTTP status 503"));
    assert!(!message.contains("sk-secret-token"));
    assert!(!message.contains("api_key"));
    assert!(!message.contains("password"));
}
