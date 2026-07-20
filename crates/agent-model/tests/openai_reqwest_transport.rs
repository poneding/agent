use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use agent_model::{
    InMemorySecretResolver, ModelEndpointConfig, ModelProvider, ModelProviderConfig,
    ModelProviderKind, ModelStreamEvent, OpenAiCompatibleHttpRequest, OpenAiCompatibleProvider,
    OpenAiCompatibleTransport, ReqwestOpenAiCompatibleTransport, SecretRef,
    collect_model_event_stream,
};
use serde_json::json;

#[derive(Clone, Debug)]
struct RecordedRequest {
    path: String,
    authorization: Option<String>,
    body: String,
}

fn secret_ref() -> SecretRef {
    SecretRef::new("env", "OPENAI_API_KEY").expect("valid secret ref")
}

fn resolver() -> InMemorySecretResolver {
    InMemorySecretResolver::new()
        .with_secret(secret_ref(), "sk-test-secret")
        .expect("secret resolver")
}

fn http_request(url: String) -> OpenAiCompatibleHttpRequest {
    OpenAiCompatibleHttpRequest {
        url,
        secret_ref: secret_ref(),
        body: json!({
            "model": "gpt-4.1-mini",
            "input": [{ "role": "user", "content": "hello" }],
            "stream": true,
        }),
        timeout_ms: 5_000,
    }
}

fn provider_config(base_url: String) -> ModelProviderConfig {
    ModelProviderConfig::new(
        "openai",
        ModelProviderKind::OpenAiCompatible,
        "gpt-4.1-mini",
        ModelEndpointConfig::new(Some(&base_url), 5_000).expect("valid endpoint"),
        Some(secret_ref()),
        true,
    )
    .expect("valid provider config")
}

fn spawn_server(
    response_status: &str,
    response_body: &str,
) -> (String, mpsc::Receiver<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local server");
    let addr = listener.local_addr().expect("local addr");
    let (sender, receiver) = mpsc::channel();
    let response_status = response_status.to_string();
    let response_body = response_body.to_string();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let request = read_http_request(&mut stream);
        sender.send(request).expect("send recorded request");
        let response = format!(
            "HTTP/1.1 {response_status}\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    (format!("http://{addr}/responses"), receiver)
}

fn read_http_request(stream: &mut TcpStream) -> RecordedRequest {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout");
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        let read = stream.read(&mut buffer).expect("read request");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        if let Some((headers, body)) = split_request(&bytes) {
            let headers = String::from_utf8_lossy(headers).to_string();
            let body = String::from_utf8_lossy(body).to_string();
            let content_length = header_value(&headers, "content-length")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            if body.len() >= content_length {
                return RecordedRequest {
                    path: request_path(&headers),
                    authorization: header_value(&headers, "authorization"),
                    body,
                };
            }
        }
    }
    panic!("request ended before complete headers and body");
}

fn split_request(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let marker = b"\r\n\r\n";
    let index = bytes
        .windows(marker.len())
        .position(|window| window == marker)?;
    Some((&bytes[..index], &bytes[index + marker.len()..]))
}

fn request_path(headers: &str) -> String {
    headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("")
        .to_string()
}

fn header_value(headers: &str, name: &str) -> Option<String> {
    headers.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.eq_ignore_ascii_case(name) {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

#[tokio::test]
async fn reqwest_transport_builds_responses_request() {
    let (url, receiver) = spawn_server(
        "200 OK",
        "event: response.completed\ndata: {\"finish_reason\":\"stop\",\"output_token_count\":0}\n\n",
    );
    let transport = ReqwestOpenAiCompatibleTransport::new(Arc::new(resolver()));

    let response = transport
        .send(http_request(url))
        .await
        .expect("send request");
    let recorded = receiver.recv().expect("record request");
    let body: serde_json::Value = serde_json::from_str(&recorded.body).expect("json body");

    assert_eq!(recorded.path, "/responses");
    assert_eq!(body["stream"], true);
    assert_eq!(body["model"], "gpt-4.1-mini");
    assert_eq!(response.request_id, "reqwest-response");
}

#[tokio::test]
async fn reqwest_transport_injects_secret_without_debug_leak() {
    let (url, receiver) = spawn_server(
        "401 Unauthorized",
        "{\"error\":\"bad token sk-test-secret api_key password\"}",
    );
    let transport = ReqwestOpenAiCompatibleTransport::new(Arc::new(resolver()));
    let debug = format!("{transport:?}");

    let error = transport
        .send(http_request(url))
        .await
        .expect_err("provider error");
    let recorded = receiver.recv().expect("record request");
    let rendered = error.to_string();

    assert_eq!(
        recorded.authorization.as_deref(),
        Some("Bearer sk-test-secret")
    );
    assert!(!debug.contains("sk-test-secret"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("password"));
    assert!(!error.retryable());
}

#[tokio::test]
async fn reqwest_transport_fails_missing_secret_before_network() {
    let transport = ReqwestOpenAiCompatibleTransport::new(Arc::new(InMemorySecretResolver::new()));

    let error = transport
        .send(http_request("http://127.0.0.1:9/responses".to_string()))
        .await
        .expect_err("missing secret");

    assert!(matches!(
        error,
        agent_model::OpenAiCompatibleError::SecretResolution { .. }
    ));
    assert!(!error.retryable());
}

#[tokio::test]
async fn reqwest_transport_marks_429_as_retryable() {
    let (url, _receiver) = spawn_server("429 Too Many Requests", "{\"error\":\"rate limit\"}");
    let transport = ReqwestOpenAiCompatibleTransport::new(Arc::new(resolver()));

    let error = transport
        .send(http_request(url))
        .await
        .expect_err("rate limit error");

    assert!(error.retryable());
}

#[tokio::test]
async fn reqwest_transport_streams_sse_chunks_into_model_events() {
    let (url, _receiver) = spawn_server(
        "200 OK",
        "event: response.output_text.delta\ndata: {\"delta\":\"hel\"}\n\n\
         event: response.output_text.delta\ndata: {\"delta\":\"lo\"}\n\n\
         event: response.completed\ndata: {\"finish_reason\":\"stop\",\"output_token_count\":2}\n\n",
    );
    let base_url = url
        .strip_suffix("/responses")
        .expect("test url has responses path")
        .to_string();
    let provider = OpenAiCompatibleProvider::new(
        provider_config(base_url),
        Arc::new(ReqwestOpenAiCompatibleTransport::new(Arc::new(resolver()))),
    )
    .expect("valid provider");

    let stream = provider
        .stream(agent_model::ModelRequest {
            provider: "openai".to_string(),
            model: "gpt-4.1-mini".to_string(),
            conversation_id: agent_types::ConversationId::new(),
            turn_id: agent_types::TurnId::new(),
            correlation_id: agent_types::CorrelationId::new(),
            messages: vec![agent_model::ModelMessage::user("hello")],
            reasoning_effort: None,
            tools: Vec::new(),
        })
        .await
        .expect("stream response");
    let events = collect_model_event_stream(stream)
        .await
        .expect("collect events");

    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "reqwest-response".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "hel".to_string(),
            },
            ModelStreamEvent::Delta {
                text: "lo".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 2,
            },
        ]
    );
}
