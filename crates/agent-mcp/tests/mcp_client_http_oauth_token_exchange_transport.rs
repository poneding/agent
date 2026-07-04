use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use agent_mcp::{
    FixtureMcpClientHttpOAuthTokenExchangeTransport, McpClientHttpOAuthTokenExchangeHttpRequest,
    McpClientHttpOAuthTokenExchangeHttpResponse, McpClientHttpOAuthTokenExchangeTransport as _,
    McpClientHttpReqwestTransport,
};

#[derive(Debug)]
struct RecordedRequest {
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

fn spawn_token_server(
    status: &'static str,
    body: &'static str,
) -> (String, Receiver<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind server");
    let addr = listener.local_addr().expect("server addr");
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let recorded = read_request(stream, status, body);
        tx.send(recorded).expect("send recorded request");
    });

    (format!("http://{addr}/oauth/token"), rx)
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

fn token_request(endpoint: impl Into<String>) -> McpClientHttpOAuthTokenExchangeHttpRequest {
    McpClientHttpOAuthTokenExchangeHttpRequest::new(
        endpoint,
        vec![
            ("grant_type".to_string(), "authorization_code".to_string()),
            ("code".to_string(), "callback-code-secret-value".to_string()),
            (
                "code_verifier".to_string(),
                "verifier-secret-token".to_string(),
            ),
        ],
    )
    .expect("request")
}

#[test]
fn mcp_client_http_oauth_token_exchange_transport_request_debug_redacts_form_values() {
    let request = token_request("https://auth.example.com/token");
    let debug = format!("{request:?}");

    assert!(debug.contains("[redacted]"));
    assert!(debug.contains("grant_type"));
    assert!(!debug.contains("callback-code-secret-value"));
    assert!(!debug.contains("verifier-secret-token"));
    assert!(!debug.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_exchange_transport_response_debug_redacts_body() {
    let response = McpClientHttpOAuthTokenExchangeHttpResponse::new(
        200,
        r#"{"access_token":"access-secret-token"}"#,
    )
    .expect("response");
    let debug = format!("{response:?}");

    assert!(debug.contains("[redacted]"));
    assert!(debug.contains("200"));
    assert!(!debug.contains("access-secret-token"));
    assert!(!debug.contains("secret-token"));
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_transport_fixture_records_and_returns_response() {
    let response = McpClientHttpOAuthTokenExchangeHttpResponse::new(
        200,
        r#"{"access_token":"access-secret-token","token_type":"Bearer"}"#,
    )
    .expect("response");
    let mut transport =
        FixtureMcpClientHttpOAuthTokenExchangeTransport::from_responses([Ok(response.clone())]);
    let request = token_request("https://auth.example.com/token");

    let actual = transport
        .exchange_token(request.clone())
        .await
        .expect("exchange");

    assert_eq!(actual, response);
    assert_eq!(transport.sent_requests(), &[request]);
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_transport_reqwest_posts_form() {
    let (url, rx) = spawn_token_server(
        "200 OK",
        r#"{"access_token":"access-secret-token","token_type":"Bearer"}"#,
    );
    let request = token_request(url);
    let mut transport = McpClientHttpReqwestTransport::new();

    let response = transport.exchange_token(request).await.expect("exchange");
    let recorded = rx.recv().expect("recorded request");

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        r#"{"access_token":"access-secret-token","token_type":"Bearer"}"#
    );
    assert_eq!(recorded.path, "/oauth/token");
    assert!(
        recorded
            .headers
            .get("content-type")
            .is_some_and(|value| value.starts_with("application/x-www-form-urlencoded"))
    );
    assert!(recorded.body.contains("grant_type=authorization_code"));
    assert!(recorded.body.contains("code=callback-code-secret-value"));
    assert!(
        recorded
            .body
            .contains("code_verifier=verifier-secret-token")
    );
}

#[test]
fn mcp_client_http_oauth_token_exchange_transport_rejects_invalid_request_without_leaking() {
    let error = McpClientHttpOAuthTokenExchangeHttpRequest::new(
        "postgres://auth.example.com/token",
        vec![("code".to_string(), "callback-code-secret-value".to_string())],
    )
    .expect_err("invalid endpoint");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token exchange"));
    assert!(!message.contains("callback-code-secret-value"));
    assert!(!message.contains("postgres"));

    let error = McpClientHttpOAuthTokenExchangeHttpRequest::new(
        "https://auth.example.com/token",
        vec![("code".to_string(), " ".to_string())],
    )
    .expect_err("invalid form");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token exchange"));
    assert!(!message.contains("code"));
}
