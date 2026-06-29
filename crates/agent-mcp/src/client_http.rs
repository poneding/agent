use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    McpEnvValue, McpError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess, McpServerConfig,
    McpTransportConfig,
};

const MAX_CLIENT_HTTP_BODY_BYTES: usize = 256 * 1024;
const MAX_CLIENT_HTTP_MESSAGE_CHARS: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientHttpResponseFrame {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpRequest {
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body_json: Value,
    pub timeout_ms: u64,
}

impl McpClientHttpRequest {
    pub fn new(
        url: impl Into<String>,
        headers: BTreeMap<String, String>,
        body_json: Value,
        timeout_ms: u64,
    ) -> Result<Self, McpError> {
        let url = url.into();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(McpError::InvalidConfig(
                "streamable http url must start with http:// or https://".to_string(),
            ));
        }
        if timeout_ms == 0 {
            return Err(McpError::InvalidConfig(
                "timeout_ms must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            url,
            headers,
            body_json,
            timeout_ms,
        })
    }

    pub fn from_config(config: &McpServerConfig, body_json: Value) -> Result<Self, McpError> {
        if !config.enabled {
            return Err(McpError::InvalidConfig(
                "MCP client HTTP server is disabled".to_string(),
            ));
        }

        let McpTransportConfig::StreamableHttp { url, headers } = &config.transport else {
            return Err(McpError::InvalidConfig(
                "MCP client HTTP transport requires streamable http config".to_string(),
            ));
        };

        let mut resolved_headers = BTreeMap::new();
        for (key, value) in headers {
            match value {
                McpEnvValue::Literal(value) => {
                    resolved_headers.insert(key.clone(), value.clone());
                }
                McpEnvValue::SecretRef(_) => {
                    return Err(McpError::InvalidConfig(
                        "MCP client HTTP headers contain unresolved secret ref".to_string(),
                    ));
                }
            }
        }

        Self::new(url.clone(), resolved_headers, body_json, config.timeout_ms)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpResponse {
    pub status: u16,
    pub body_json: Value,
}

impl McpClientHttpResponse {
    pub fn new(status: u16, body_json: Value) -> Result<Self, McpError> {
        validate_mcp_client_http_status(status)?;
        Ok(Self { status, body_json })
    }
}

#[async_trait]
pub trait McpClientHttpTransport: Send {
    async fn send_request(
        &mut self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponse, McpError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClientHttpTransport {
    inbound_responses: VecDeque<Result<McpClientHttpResponse, McpError>>,
    sent_requests: Vec<McpClientHttpRequest>,
}

impl FixtureMcpClientHttpTransport {
    #[must_use]
    pub fn from_responses<I>(responses: I) -> Self
    where
        I: IntoIterator<Item = Result<McpClientHttpResponse, McpError>>,
    {
        Self {
            inbound_responses: responses.into_iter().collect(),
            sent_requests: Vec::new(),
        }
    }

    #[must_use]
    pub fn sent_requests(&self) -> &[McpClientHttpRequest] {
        &self.sent_requests
    }
}

#[async_trait]
impl McpClientHttpTransport for FixtureMcpClientHttpTransport {
    async fn send_request(
        &mut self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponse, McpError> {
        self.sent_requests.push(request);
        self.inbound_responses.pop_front().unwrap_or_else(|| {
            Err(McpError::Client(
                "MCP client HTTP fixture response missing".to_string(),
            ))
        })
    }
}

#[derive(Clone)]
pub struct McpClientHttpReqwestTransport {
    client: reqwest::Client,
}

impl McpClientHttpReqwestTransport {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    #[must_use]
    pub const fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for McpClientHttpReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for McpClientHttpReqwestTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpClientHttpReqwestTransport")
            .field("client", &"<reqwest client>")
            .finish()
    }
}

#[async_trait]
impl McpClientHttpTransport for McpClientHttpReqwestTransport {
    async fn send_request(
        &mut self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponse, McpError> {
        let mut builder = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .json(&request.body_json);
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        let response = builder.send().await.map_err(|_| {
            McpError::Client("MCP client HTTP request transport failed".to_string())
        })?;
        let status = response.status().as_u16();
        validate_mcp_client_http_status(status)?;

        let body = response
            .text()
            .await
            .map_err(|_| McpError::Client("MCP client HTTP response read failed".to_string()))?;
        if body.len() > MAX_CLIENT_HTTP_BODY_BYTES {
            return Err(McpError::InvalidServerRequest(
                "MCP client HTTP response body too large".to_string(),
            ));
        }
        let body_json = serde_json::from_str::<Value>(&body).map_err(|_| {
            McpError::InvalidServerRequest("invalid MCP client HTTP response body".to_string())
        })?;

        Ok(McpClientHttpResponse { status, body_json })
    }
}

impl McpClientHttpResponseFrame {
    #[must_use]
    pub const fn id(&self) -> &McpJsonRpcId {
        match self {
            Self::Success(success) => &success.id,
            Self::Failure(failure) => &failure.id,
        }
    }
}

pub fn render_mcp_client_http_request_body(
    id: McpJsonRpcId,
    method: impl Into<String>,
    params: Value,
) -> Result<Value, McpError> {
    if matches!(id, McpJsonRpcId::Null) {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP request id cannot be null".to_string(),
        ));
    }

    let method = method.into();
    if method.trim().is_empty() {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP method cannot be empty".to_string(),
        ));
    }

    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
}

pub fn parse_mcp_client_http_response_text(
    body: &str,
) -> Result<McpClientHttpResponseFrame, McpError> {
    if body.len() > MAX_CLIENT_HTTP_BODY_BYTES {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP response body too large".to_string(),
        ));
    }

    let value = serde_json::from_str::<Value>(body).map_err(|_| {
        McpError::InvalidServerRequest("invalid MCP client HTTP response body".to_string())
    })?;
    parse_mcp_client_http_response_body(&value)
}

pub fn parse_mcp_client_http_response_body(
    body: &Value,
) -> Result<McpClientHttpResponseFrame, McpError> {
    if body.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP response JSON-RPC version must be 2.0".to_string(),
        ));
    }
    if body.get("id").is_none() {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP response id missing".to_string(),
        ));
    }

    match (body.get("result").is_some(), body.get("error").is_some()) {
        (true, false) => serde_json::from_value::<McpJsonRpcSuccess>(body.clone())
            .map(McpClientHttpResponseFrame::Success)
            .map_err(|_| {
                McpError::InvalidServerRequest(
                    "invalid MCP client HTTP success response".to_string(),
                )
            }),
        (false, true) => {
            let mut failure =
                serde_json::from_value::<McpJsonRpcFailure>(body.clone()).map_err(|_| {
                    McpError::InvalidServerRequest(
                        "invalid MCP client HTTP failure response".to_string(),
                    )
                })?;
            failure.error.message = public_http_message(&failure.error.message);
            Ok(McpClientHttpResponseFrame::Failure(failure))
        }
        _ => Err(McpError::InvalidServerRequest(
            "MCP client HTTP response must contain exactly one result or error".to_string(),
        )),
    }
}

pub fn validate_mcp_client_http_status(status: u16) -> Result<(), McpError> {
    if (200..300).contains(&status) {
        return Ok(());
    }

    Err(McpError::Client(format!(
        "MCP client HTTP request failed with HTTP status {status}"
    )))
}

fn public_http_message(value: &str) -> String {
    truncate_chars(
        &redact_secret_like_text(value),
        MAX_CLIENT_HTTP_MESSAGE_CHARS,
    )
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }

    let prefix = value.chars().take(max_chars - 3).collect::<String>();
    format!("{prefix}...")
}

fn redact_secret_like_text(value: &str) -> String {
    if is_secret_like(value) {
        "[redacted]".to_string()
    } else {
        value.to_string()
    }
}

fn is_secret_like(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace(['-', '.'], "_");
    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("private_key")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("sk_")
}
