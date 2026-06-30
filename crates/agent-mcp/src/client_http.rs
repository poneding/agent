use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    MCP_SERVER_PROTOCOL_VERSION, McpClient, McpClientRootProvider, McpEnvValue, McpError,
    McpGatewayClient, McpInitializeResult, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess,
    McpRootListResult, McpServerCapabilities, McpServerConfig, McpServerId, McpServerMethod,
    McpToolCallRequest, McpToolCallResult, McpToolDescriptor, McpToolListResult,
    McpTransportConfig,
};

const MAX_CLIENT_HTTP_BODY_BYTES: usize = 256 * 1024;
const MAX_CLIENT_HTTP_MESSAGE_CHARS: usize = 256;
const MAX_CLIENT_HTTP_METADATA_VALUE_CHARS: usize = 256;
const MAX_CLIENT_HTTP_SSE_EVENT_DATA_BYTES: usize = 64 * 1024;

const MCP_CLIENT_HTTP_SESSION_ID_HEADER_CANONICAL: &str = "Mcp-Session-Id";
const MCP_CLIENT_HTTP_PROTOCOL_VERSION_HEADER_CANONICAL: &str = "MCP-Protocol-Version";
const MCP_CLIENT_HTTP_LAST_EVENT_ID_HEADER_CANONICAL: &str = "Last-Event-ID";
const MCP_CLIENT_HTTP_SESSION_ID_HEADER: &str = "mcp-session-id";
const MCP_CLIENT_HTTP_PROTOCOL_VERSION_HEADER: &str = "mcp-protocol-version";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientHttpResponseFrame {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpSseEvent {
    pub event: Option<String>,
    pub id: Option<String>,
    pub retry_ms: Option<u64>,
    pub data: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpSseResponse {
    pub events: Vec<McpClientHttpSseEvent>,
}

impl McpClientHttpSseResponse {
    #[must_use]
    pub const fn new(events: Vec<McpClientHttpSseEvent>) -> Self {
        Self { events }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpSessionMetadata {
    pub session_id: Option<String>,
    pub protocol_version: Option<String>,
    pub last_event_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpResponseEnvelope {
    pub mode: McpClientHttpResponseMode,
    pub metadata: McpClientHttpSessionMetadata,
}

impl McpClientHttpResponseEnvelope {
    #[must_use]
    pub const fn new(
        mode: McpClientHttpResponseMode,
        metadata: McpClientHttpSessionMetadata,
    ) -> Self {
        Self { mode, metadata }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "response")]
pub enum McpClientHttpResponseMode {
    Json(McpClientHttpResponse),
    Sse(McpClientHttpSseResponse),
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
        Self::from_config_with_session_metadata(config, body_json, None)
    }

    pub fn from_config_with_session_metadata(
        config: &McpServerConfig,
        body_json: Value,
        session_metadata: Option<&McpClientHttpSessionMetadata>,
    ) -> Result<Self, McpError> {
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
        if let Some(metadata) = session_metadata {
            insert_mcp_client_http_metadata_header(
                &mut resolved_headers,
                MCP_CLIENT_HTTP_SESSION_ID_HEADER_CANONICAL,
                metadata.session_id.as_deref(),
            )?;
            insert_mcp_client_http_metadata_header(
                &mut resolved_headers,
                MCP_CLIENT_HTTP_PROTOCOL_VERSION_HEADER_CANONICAL,
                metadata.protocol_version.as_deref(),
            )?;
        }

        Self::new(url.clone(), resolved_headers, body_json, config.timeout_ms)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpStreamRequest {
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub timeout_ms: u64,
}

impl McpClientHttpStreamRequest {
    pub fn new(
        url: impl Into<String>,
        headers: BTreeMap<String, String>,
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
            timeout_ms,
        })
    }

    pub fn from_config(config: &McpServerConfig) -> Result<Self, McpError> {
        Self::from_config_with_session_metadata(config, &McpClientHttpSessionMetadata::default())
    }

    pub fn from_config_with_session_metadata(
        config: &McpServerConfig,
        session_metadata: &McpClientHttpSessionMetadata,
    ) -> Result<Self, McpError> {
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

        insert_mcp_client_http_metadata_header(
            &mut resolved_headers,
            MCP_CLIENT_HTTP_SESSION_ID_HEADER_CANONICAL,
            session_metadata.session_id.as_deref(),
        )?;
        insert_mcp_client_http_metadata_header(
            &mut resolved_headers,
            MCP_CLIENT_HTTP_PROTOCOL_VERSION_HEADER_CANONICAL,
            session_metadata.protocol_version.as_deref(),
        )?;
        insert_mcp_client_http_metadata_header(
            &mut resolved_headers,
            MCP_CLIENT_HTTP_LAST_EVENT_ID_HEADER_CANONICAL,
            session_metadata.last_event_id.as_deref(),
        )?;

        Self::new(url.clone(), resolved_headers, config.timeout_ms)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpStreamOpen {
    pub metadata: McpClientHttpSessionMetadata,
    pub events: Vec<McpClientHttpSseEvent>,
}

impl McpClientHttpStreamOpen {
    #[must_use]
    pub const fn new(
        metadata: McpClientHttpSessionMetadata,
        events: Vec<McpClientHttpSseEvent>,
    ) -> Self {
        Self { metadata, events }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpStreamDrainReport {
    pub accepted_events: usize,
    pub last_event_id: Option<String>,
}

impl McpClientHttpStreamDrainReport {
    #[must_use]
    pub const fn new(accepted_events: usize, last_event_id: Option<String>) -> Self {
        Self {
            accepted_events,
            last_event_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientHttpReconnectPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for McpClientHttpReconnectPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 250,
            max_backoff_ms: 4_000,
        }
    }
}

impl McpClientHttpReconnectPolicy {
    pub fn validate(&self) -> Result<(), McpError> {
        if self.max_attempts == 0
            || self.initial_backoff_ms == 0
            || self.max_backoff_ms == 0
            || self.initial_backoff_ms > self.max_backoff_ms
        {
            return Err(invalid_mcp_client_http_reconnect_policy());
        }
        Ok(())
    }

    pub fn decision_for_attempt(
        &self,
        attempt: u32,
        retryable: bool,
    ) -> Result<McpClientHttpReconnectDecision, McpError> {
        self.validate()?;
        if !retryable || attempt >= self.max_attempts {
            return Ok(McpClientHttpReconnectDecision::Stop);
        }

        Ok(McpClientHttpReconnectDecision::Retry {
            backoff_ms: self.backoff_for_attempt(attempt),
        })
    }

    #[must_use]
    pub fn backoff_for_attempt(&self, attempt: u32) -> u64 {
        let multiplier = attempt.saturating_sub(1);
        let mut backoff_ms = self.initial_backoff_ms;
        for _ in 0..multiplier {
            backoff_ms = backoff_ms.saturating_mul(2).min(self.max_backoff_ms);
        }
        backoff_ms.min(self.max_backoff_ms)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientHttpReconnectDecision {
    Retry { backoff_ms: u64 },
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientHttpReconnectFailureKind {
    RetryableTransport,
    RetryableStatus,
    NonRetryableConfig,
    NonRetryableValidation,
    NonRetryableOther,
}

impl McpClientHttpReconnectFailureKind {
    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::RetryableTransport | Self::RetryableStatus)
    }

    #[must_use]
    pub const fn public_reason(self) -> &'static str {
        match self {
            Self::RetryableTransport => "retryable MCP client HTTP transport failure",
            Self::RetryableStatus => "retryable MCP client HTTP status failure",
            Self::NonRetryableConfig => "non-retryable MCP client HTTP config failure",
            Self::NonRetryableValidation => "non-retryable MCP client HTTP validation failure",
            Self::NonRetryableOther => "non-retryable MCP client HTTP failure",
        }
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

impl From<McpClientHttpResponse> for McpClientHttpResponseMode {
    fn from(response: McpClientHttpResponse) -> Self {
        Self::Json(response)
    }
}

impl From<McpClientHttpResponse> for McpClientHttpResponseEnvelope {
    fn from(response: McpClientHttpResponse) -> Self {
        Self::new(response.into(), McpClientHttpSessionMetadata::default())
    }
}

impl From<McpClientHttpSseResponse> for McpClientHttpResponseMode {
    fn from(response: McpClientHttpSseResponse) -> Self {
        Self::Sse(response)
    }
}

impl From<McpClientHttpSseResponse> for McpClientHttpResponseEnvelope {
    fn from(response: McpClientHttpSseResponse) -> Self {
        Self::new(response.into(), McpClientHttpSessionMetadata::default())
    }
}

impl From<McpClientHttpResponseMode> for McpClientHttpResponseEnvelope {
    fn from(mode: McpClientHttpResponseMode) -> Self {
        Self::new(mode, McpClientHttpSessionMetadata::default())
    }
}

#[async_trait]
pub trait McpClientHttpTransport: Send {
    async fn send_request(
        &mut self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponseEnvelope, McpError>;
}

#[async_trait]
pub trait McpClientHttpStreamTransport: Send {
    async fn open_stream(
        &mut self,
        request: McpClientHttpStreamRequest,
    ) -> Result<McpClientHttpStreamOpen, McpError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClientHttpTransport {
    inbound_responses: VecDeque<Result<McpClientHttpResponseEnvelope, McpError>>,
    sent_requests: Vec<McpClientHttpRequest>,
}

impl FixtureMcpClientHttpTransport {
    #[must_use]
    pub fn from_responses<I, R>(responses: I) -> Self
    where
        I: IntoIterator<Item = Result<R, McpError>>,
        R: Into<McpClientHttpResponseEnvelope>,
    {
        Self {
            inbound_responses: responses
                .into_iter()
                .map(|response| response.map(Into::into))
                .collect(),
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
    ) -> Result<McpClientHttpResponseEnvelope, McpError> {
        self.sent_requests.push(request);
        self.inbound_responses.pop_front().unwrap_or_else(|| {
            Err(McpError::Client(
                "MCP client HTTP fixture response missing".to_string(),
            ))
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClientHttpStreamTransport {
    inbound_streams: VecDeque<Result<McpClientHttpStreamOpen, McpError>>,
    sent_requests: Vec<McpClientHttpStreamRequest>,
}

impl FixtureMcpClientHttpStreamTransport {
    #[must_use]
    pub fn from_streams<I>(streams: I) -> Self
    where
        I: IntoIterator<Item = Result<McpClientHttpStreamOpen, McpError>>,
    {
        Self {
            inbound_streams: streams.into_iter().collect(),
            sent_requests: Vec::new(),
        }
    }

    #[must_use]
    pub fn sent_requests(&self) -> &[McpClientHttpStreamRequest] {
        &self.sent_requests
    }
}

#[async_trait]
impl McpClientHttpStreamTransport for FixtureMcpClientHttpStreamTransport {
    async fn open_stream(
        &mut self,
        request: McpClientHttpStreamRequest,
    ) -> Result<McpClientHttpStreamOpen, McpError> {
        self.sent_requests.push(request);
        self.inbound_streams.pop_front().unwrap_or_else(|| {
            Err(McpError::Client(
                "MCP client HTTP fixture stream missing".to_string(),
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
    ) -> Result<McpClientHttpResponseEnvelope, McpError> {
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
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let metadata_headers = mcp_client_http_metadata_headers(response.headers());

        let body = response
            .text()
            .await
            .map_err(|_| McpError::Client("MCP client HTTP response read failed".to_string()))?;
        if body.len() > MAX_CLIENT_HTTP_BODY_BYTES {
            return Err(McpError::InvalidServerRequest(
                "MCP client HTTP response body too large".to_string(),
            ));
        }

        if content_type
            .as_deref()
            .is_some_and(is_mcp_client_http_sse_content_type)
        {
            let events = parse_mcp_client_http_sse_events(&body)?;
            let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));
            let metadata = parse_mcp_client_http_session_metadata(&metadata_headers, &mode)?;
            return Ok(McpClientHttpResponseEnvelope::new(mode, metadata));
        }
        if content_type
            .as_deref()
            .is_some_and(|content_type| !is_mcp_client_http_json_content_type(content_type))
        {
            return Err(McpError::InvalidServerRequest(
                "unsupported MCP client HTTP response content type".to_string(),
            ));
        }

        let body_json = serde_json::from_str::<Value>(&body).map_err(|_| {
            McpError::InvalidServerRequest("invalid MCP client HTTP response body".to_string())
        })?;

        let mode = McpClientHttpResponseMode::Json(McpClientHttpResponse { status, body_json });
        let metadata = parse_mcp_client_http_session_metadata(&metadata_headers, &mode)?;

        Ok(McpClientHttpResponseEnvelope::new(mode, metadata))
    }
}

#[async_trait]
impl McpClientHttpStreamTransport for McpClientHttpReqwestTransport {
    async fn open_stream(
        &mut self,
        request: McpClientHttpStreamRequest,
    ) -> Result<McpClientHttpStreamOpen, McpError> {
        let mut builder = self
            .client
            .get(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms));
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        let response = builder
            .send()
            .await
            .map_err(|_| McpError::Client("MCP client HTTP stream transport failed".to_string()))?;
        let status = response.status().as_u16();
        validate_mcp_client_http_status(status)?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let metadata_headers = mcp_client_http_metadata_headers(response.headers());

        if !content_type
            .as_deref()
            .is_some_and(is_mcp_client_http_sse_content_type)
        {
            return Err(McpError::InvalidServerRequest(
                "unsupported MCP client HTTP stream content type".to_string(),
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|_| McpError::Client("MCP client HTTP stream read failed".to_string()))?;
        if body.len() > MAX_CLIENT_HTTP_BODY_BYTES {
            return Err(McpError::InvalidServerRequest(
                "MCP client HTTP stream body too large".to_string(),
            ));
        }

        let events = parse_mcp_client_http_sse_events(&body)?;
        let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events.clone()));
        let metadata = parse_mcp_client_http_session_metadata(&metadata_headers, &mode)?;

        Ok(McpClientHttpStreamOpen::new(metadata, events))
    }
}

pub struct McpHttpClient<T> {
    state: Mutex<McpHttpClientState<T>>,
}

struct McpHttpClientState<T> {
    transport: T,
    server_id: Option<McpServerId>,
    config: Option<McpServerConfig>,
    next_request_id: i64,
    session_metadata: McpClientHttpSessionMetadata,
}

impl<T> McpHttpClient<T> {
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self {
            state: Mutex::new(McpHttpClientState {
                transport,
                server_id: None,
                config: None,
                next_request_id: 1,
                session_metadata: McpClientHttpSessionMetadata::default(),
            }),
        }
    }

    pub async fn session_metadata_snapshot(&self) -> McpClientHttpSessionMetadata {
        let state = self.state.lock().await;
        state.session_metadata.clone()
    }

    pub async fn stream_request_snapshot(
        &self,
        server_id: &McpServerId,
    ) -> Result<McpClientHttpStreamRequest, McpError> {
        let state = self.state.lock().await;
        if state.server_id.as_ref() != Some(server_id) {
            return Err(McpError::Client(
                "MCP client HTTP session is not initialized".to_string(),
            ));
        }
        let config = state.config.clone().ok_or_else(|| {
            McpError::Client("MCP client HTTP session is not initialized".to_string())
        })?;
        McpClientHttpStreamRequest::from_config_with_session_metadata(
            &config,
            &state.session_metadata,
        )
    }

    pub async fn record_stream_open(
        &self,
        server_id: &McpServerId,
        stream: McpClientHttpStreamOpen,
    ) -> Result<McpClientHttpStreamDrainReport, McpError> {
        let mut state = self.state.lock().await;
        if state.server_id.as_ref() != Some(server_id) {
            return Err(McpError::Client(
                "MCP client HTTP session is not initialized".to_string(),
            ));
        }

        let mut accepted_events = 0usize;
        let mut last_event_id = None;
        for event in &stream.events {
            let _frame = parse_mcp_client_http_response_text(&event.data)?;
            if let Some(event_id) = &event.id {
                last_event_id = Some(validate_mcp_client_http_metadata_value(event_id)?);
            }
            accepted_events += 1;
        }

        if let Some(event_id) = &last_event_id {
            state.session_metadata.last_event_id = Some(event_id.clone());
        }

        Ok(McpClientHttpStreamDrainReport::new(
            accepted_events,
            last_event_id,
        ))
    }
}

impl McpHttpClient<McpClientHttpReqwestTransport> {
    #[must_use]
    pub fn reqwest() -> Self {
        Self::new(McpClientHttpReqwestTransport::new())
    }
}

impl<T> McpHttpClientState<T>
where
    T: McpClientHttpTransport,
{
    async fn send_request(
        &mut self,
        config: &McpServerConfig,
        method: &str,
        params: Value,
    ) -> Result<McpClientHttpDispatchResult, McpError> {
        let id = self.allocate_request_id()?;
        let body = render_mcp_client_http_request_body(id.clone(), method, params)?;
        let request = McpClientHttpRequest::from_config_with_session_metadata(
            config,
            body,
            Some(&self.session_metadata),
        )?;
        let envelope = self.transport.send_request(request).await?;
        let frame = find_mcp_client_http_response_frame(&envelope.mode, &id)?;
        Ok(McpClientHttpDispatchResult {
            frame,
            metadata: envelope.metadata,
        })
    }

    fn allocate_request_id(&mut self) -> Result<McpJsonRpcId, McpError> {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.checked_add(1).ok_or_else(|| {
            McpError::InvalidServerRequest("MCP client HTTP request id exhausted".to_string())
        })?;
        Ok(McpJsonRpcId::Number(id))
    }

    fn record_accepted_response_metadata(&mut self, metadata: &McpClientHttpSessionMetadata) {
        if let Some(last_event_id) = &metadata.last_event_id {
            self.session_metadata.last_event_id = Some(last_event_id.clone());
        }
    }
}

#[async_trait]
impl<T> McpClient for McpHttpClient<T>
where
    T: McpClientHttpTransport,
{
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError> {
        let mut state = self.state.lock().await;
        let dispatch = state
            .send_request(
                config,
                McpServerMethod::Initialize.as_str(),
                json!({
                    "protocolVersion": MCP_SERVER_PROTOCOL_VERSION,
                    "clientInfo": {
                        "name": "agent",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "capabilities": {},
                }),
            )
            .await?;
        let result = mcp_client_http_success_result(dispatch.frame, "initialize")?;
        let initialize = map_mcp_client_http_initialize_result(config.id.clone(), &result)?;
        state.server_id = Some(config.id.clone());
        state.config = Some(config.clone());
        state.session_metadata = dispatch.metadata;
        Ok(initialize)
    }

    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError> {
        let mut state = self.state.lock().await;
        if state.server_id.as_ref() != Some(server_id) {
            return Err(McpError::Client(
                "MCP client HTTP session is not initialized".to_string(),
            ));
        }
        let config = state.config.clone().ok_or_else(|| {
            McpError::Client("MCP client HTTP session is not initialized".to_string())
        })?;

        let dispatch = state
            .send_request(&config, McpServerMethod::ToolsList.as_str(), json!({}))
            .await?;
        let result = mcp_client_http_success_result(dispatch.frame, "tools/list")?;
        let tools = map_mcp_client_http_tool_list_result(server_id.clone(), &result)?;
        state.record_accepted_response_metadata(&dispatch.metadata);
        Ok(tools)
    }
}

#[async_trait]
impl<T> McpClientRootProvider for McpHttpClient<T>
where
    T: McpClientHttpTransport,
{
    async fn list_roots(&self, server_id: &McpServerId) -> Result<McpRootListResult, McpError> {
        let state = self.state.lock().await;
        if state.server_id.as_ref() != Some(server_id) {
            return Err(McpError::Client(
                "MCP client HTTP session is not initialized".to_string(),
            ));
        }
        Ok(McpRootListResult::empty())
    }
}

#[async_trait]
impl<T> McpGatewayClient for McpHttpClient<T>
where
    T: McpClientHttpTransport,
{
    async fn call_tool(&self, request: McpToolCallRequest) -> Result<McpToolCallResult, McpError> {
        let mut state = self.state.lock().await;
        if state.server_id.as_ref() != Some(&request.server_id) {
            return Err(McpError::Client(
                "MCP client HTTP session is not initialized".to_string(),
            ));
        }
        let config = state.config.clone().ok_or_else(|| {
            McpError::Client("MCP client HTTP session is not initialized".to_string())
        })?;

        let dispatch = state
            .send_request(
                &config,
                McpServerMethod::ToolsCall.as_str(),
                json!({
                    "name": request.remote_tool_name,
                    "arguments": request.input_json,
                }),
            )
            .await?;
        let result = mcp_client_http_success_result(dispatch.frame, "tools/call")?;
        let output = map_mcp_client_http_tool_call_result(&result)?;
        state.record_accepted_response_metadata(&dispatch.metadata);
        Ok(output)
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

#[must_use]
pub fn classify_mcp_client_http_reconnect_error(
    error: &McpError,
) -> McpClientHttpReconnectFailureKind {
    match error {
        McpError::InvalidConfig(_) => McpClientHttpReconnectFailureKind::NonRetryableConfig,
        McpError::InvalidServerRequest(_) => {
            McpClientHttpReconnectFailureKind::NonRetryableValidation
        }
        McpError::Client(message) if is_retryable_mcp_client_http_transport_error(message) => {
            McpClientHttpReconnectFailureKind::RetryableTransport
        }
        McpError::Client(message) if retryable_mcp_client_http_status(message) => {
            McpClientHttpReconnectFailureKind::RetryableStatus
        }
        McpError::Client(_) => McpClientHttpReconnectFailureKind::NonRetryableOther,
        McpError::InvalidToolDescriptor(_) | McpError::ServerNotFound(_) => {
            McpClientHttpReconnectFailureKind::NonRetryableOther
        }
    }
}

pub fn parse_mcp_client_http_sse_events(
    body: &str,
) -> Result<Vec<McpClientHttpSseEvent>, McpError> {
    if body.len() > MAX_CLIENT_HTTP_BODY_BYTES {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP SSE response body too large".to_string(),
        ));
    }

    let mut events = Vec::new();
    let mut pending = PendingMcpClientHttpSseEvent::default();
    for raw_line in body.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() {
            flush_mcp_client_http_sse_event(&mut pending, &mut events)?;
            continue;
        }
        if line.starts_with(':') {
            continue;
        }

        let (field, value) = line.split_once(':').map_or((line, ""), |(field, value)| {
            (field, value.strip_prefix(' ').unwrap_or(value))
        });
        match field {
            "data" => pending.push_data(value)?,
            "event" => pending.event = non_empty_sse_value(value),
            "id" => pending.id = non_empty_sse_value(value),
            "retry" => {
                if let Ok(retry_ms) = value.parse::<u64>() {
                    pending.retry_ms = Some(retry_ms);
                }
            }
            _ => {}
        }
    }
    flush_mcp_client_http_sse_event(&mut pending, &mut events)?;

    Ok(events)
}

pub fn parse_mcp_client_http_response_mode_frames(
    response: &McpClientHttpResponseMode,
) -> Result<Vec<McpClientHttpResponseFrame>, McpError> {
    match response {
        McpClientHttpResponseMode::Json(response) => {
            parse_mcp_client_http_response_body(&response.body_json).map(|frame| vec![frame])
        }
        McpClientHttpResponseMode::Sse(response) => response
            .events
            .iter()
            .map(|event| parse_mcp_client_http_response_text(&event.data))
            .collect(),
    }
}

pub fn find_mcp_client_http_response_frame(
    response: &McpClientHttpResponseMode,
    id: &McpJsonRpcId,
) -> Result<McpClientHttpResponseFrame, McpError> {
    parse_mcp_client_http_response_mode_frames(response)?
        .into_iter()
        .find(|frame| frame.id() == id)
        .ok_or_else(|| {
            McpError::InvalidServerRequest(
                "MCP client HTTP matching response id missing".to_string(),
            )
        })
}

pub fn parse_mcp_client_http_session_metadata(
    headers: &BTreeMap<String, String>,
    response: &McpClientHttpResponseMode,
) -> Result<McpClientHttpSessionMetadata, McpError> {
    Ok(McpClientHttpSessionMetadata {
        session_id: parse_mcp_client_http_metadata_header(
            headers,
            MCP_CLIENT_HTTP_SESSION_ID_HEADER,
        )?,
        protocol_version: parse_mcp_client_http_metadata_header(
            headers,
            MCP_CLIENT_HTTP_PROTOCOL_VERSION_HEADER,
        )?,
        last_event_id: parse_mcp_client_http_last_sse_event_id(response)?,
    })
}

fn mcp_client_http_success_result(
    response: McpClientHttpResponseFrame,
    operation: &str,
) -> Result<Value, McpError> {
    match response {
        McpClientHttpResponseFrame::Success(success) => Ok(success.result),
        McpClientHttpResponseFrame::Failure(failure) => Err(McpError::Client(format!(
            "MCP client HTTP {operation} failed with code {}",
            failure.error.code
        ))),
    }
}

fn map_mcp_client_http_initialize_result(
    server_id: McpServerId,
    result: &Value,
) -> Result<McpInitializeResult, McpError> {
    let protocol_version = string_field_any(
        result,
        &["protocol_version", "protocolVersion"],
        "initialize",
    )?;
    let server_info = object_field_any(result, &["server_info", "serverInfo"], "initialize")?;
    let server_name = string_field(server_info, "name", "initialize")?;
    let server_version = optional_string_field(server_info, "version", "initialize")?;
    let capabilities = object_field(result, "capabilities", "initialize")?;

    Ok(McpInitializeResult::new(
        server_id,
        protocol_version,
        server_name,
        server_version,
        McpServerCapabilities {
            tools: capability_enabled(capabilities.get("tools")),
            prompts: capability_enabled(capabilities.get("prompts")),
            resources: capability_enabled(capabilities.get("resources")),
        },
    ))
}

fn map_mcp_client_http_tool_list_result(
    server_id: McpServerId,
    result: &Value,
) -> Result<McpToolListResult, McpError> {
    let tools = result
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_mcp_client_http_result("tools/list"))?;
    let mut descriptors = Vec::with_capacity(tools.len());
    for tool in tools {
        let tool = tool
            .as_object()
            .ok_or_else(|| invalid_mcp_client_http_result("tools/list"))?;
        let name = string_field(tool, "name", "tools/list")?;
        let title = optional_string_field(tool, "title", "tools/list")?;
        let description = string_field(tool, "description", "tools/list")?;
        let input_schema = value_field_any(tool, &["input_schema", "inputSchema"], "tools/list")?;
        descriptors.push(McpToolDescriptor::new(
            server_id.clone(),
            name,
            title,
            description,
            input_schema.clone(),
        )?);
    }
    Ok(McpToolListResult::new(server_id, descriptors))
}

fn map_mcp_client_http_tool_call_result(result: &Value) -> Result<McpToolCallResult, McpError> {
    let result_object = result
        .as_object()
        .ok_or_else(|| invalid_mcp_client_http_result("tools/call"))?;

    if result_object.contains_key("output_json") {
        return serde_json::from_value::<McpToolCallResult>(result.clone())
            .map_err(|_| invalid_mcp_client_http_result("tools/call"));
    }

    Ok(McpToolCallResult::new(result.clone()))
}

fn object_field<'a>(
    value: &'a Value,
    field: &str,
    operation: &str,
) -> Result<&'a serde_json::Map<String, Value>, McpError> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_mcp_client_http_result(operation))
}

fn object_field_any<'a>(
    value: &'a Value,
    fields: &[&str],
    operation: &str,
) -> Result<&'a serde_json::Map<String, Value>, McpError> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(Value::as_object))
        .ok_or_else(|| invalid_mcp_client_http_result(operation))
}

fn value_field_any<'a>(
    value: &'a serde_json::Map<String, Value>,
    fields: &[&str],
    operation: &str,
) -> Result<&'a Value, McpError> {
    fields
        .iter()
        .find_map(|field| value.get(*field))
        .ok_or_else(|| invalid_mcp_client_http_result(operation))
}

fn string_field(
    value: &serde_json::Map<String, Value>,
    field: &str,
    operation: &str,
) -> Result<String, McpError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| invalid_mcp_client_http_result(operation))
}

fn string_field_any(value: &Value, fields: &[&str], operation: &str) -> Result<String, McpError> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(Value::as_str))
        .filter(|text| !text.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| invalid_mcp_client_http_result(operation))
}

fn optional_string_field(
    value: &serde_json::Map<String, Value>,
    field: &str,
    operation: &str,
) -> Result<Option<String>, McpError> {
    match value.get(field) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text.clone())),
        Some(Value::String(_)) | Some(Value::Null) | None => Ok(None),
        Some(_) => Err(invalid_mcp_client_http_result(operation)),
    }
}

fn capability_enabled(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(enabled)) => *enabled,
        Some(Value::Object(_)) => true,
        _ => false,
    }
}

fn invalid_mcp_client_http_result(operation: &str) -> McpError {
    McpError::Client(format!("invalid MCP client HTTP {operation} result"))
}

fn public_http_message(value: &str) -> String {
    truncate_chars(
        &redact_secret_like_text(value),
        MAX_CLIENT_HTTP_MESSAGE_CHARS,
    )
}

struct McpClientHttpDispatchResult {
    frame: McpClientHttpResponseFrame,
    metadata: McpClientHttpSessionMetadata,
}

fn is_mcp_client_http_sse_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/event-stream"))
}

fn is_mcp_client_http_json_content_type(content_type: &str) -> bool {
    content_type.split(';').next().is_some_and(|value| {
        let value = value.trim().to_ascii_lowercase();
        value == "application/json" || value.ends_with("+json")
    })
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

fn mcp_client_http_metadata_headers(
    headers: &reqwest::header::HeaderMap,
) -> BTreeMap<String, String> {
    [
        MCP_CLIENT_HTTP_SESSION_ID_HEADER,
        MCP_CLIENT_HTTP_PROTOCOL_VERSION_HEADER,
    ]
    .into_iter()
    .filter_map(|name| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| (name.to_string(), value.to_string()))
    })
    .collect()
}

fn insert_mcp_client_http_metadata_header(
    headers: &mut BTreeMap<String, String>,
    name: &str,
    value: Option<&str>,
) -> Result<(), McpError> {
    let Some(value) = value else {
        return Ok(());
    };
    if headers.keys().any(|key| key.eq_ignore_ascii_case(name)) {
        return Err(McpError::InvalidConfig(
            "MCP client HTTP session header conflicts with configured header".to_string(),
        ));
    }
    headers.insert(
        name.to_string(),
        validate_mcp_client_http_metadata_value(value)?,
    );
    Ok(())
}

fn parse_mcp_client_http_metadata_header(
    headers: &BTreeMap<String, String>,
    expected_name: &str,
) -> Result<Option<String>, McpError> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(expected_name))
        .map(|(_, value)| validate_mcp_client_http_metadata_value(value))
        .transpose()
}

fn parse_mcp_client_http_last_sse_event_id(
    response: &McpClientHttpResponseMode,
) -> Result<Option<String>, McpError> {
    let McpClientHttpResponseMode::Sse(response) = response else {
        return Ok(None);
    };

    response
        .events
        .iter()
        .rev()
        .find_map(|event| event.id.as_deref())
        .map(validate_mcp_client_http_metadata_value)
        .transpose()
}

fn validate_mcp_client_http_metadata_value(value: &str) -> Result<String, McpError> {
    if value.chars().any(char::is_control) {
        return Err(invalid_mcp_client_http_session_metadata());
    }

    let value = value.trim();
    if value.is_empty()
        || value.chars().count() > MAX_CLIENT_HTTP_METADATA_VALUE_CHARS
        || value.chars().any(char::is_control)
    {
        return Err(invalid_mcp_client_http_session_metadata());
    }

    Ok(value.to_string())
}

fn invalid_mcp_client_http_session_metadata() -> McpError {
    McpError::InvalidServerRequest("invalid MCP client HTTP session metadata".to_string())
}

fn invalid_mcp_client_http_reconnect_policy() -> McpError {
    McpError::InvalidConfig("invalid MCP client HTTP reconnect policy".to_string())
}

fn is_retryable_mcp_client_http_transport_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("transport failed") || message.contains("stream read failed")
}

fn retryable_mcp_client_http_status(message: &str) -> bool {
    let Some(status) = parse_mcp_client_http_status_from_message(message) else {
        return false;
    };
    status == 408 || status == 429 || status >= 500
}

fn parse_mcp_client_http_status_from_message(message: &str) -> Option<u16> {
    let (_, suffix) = message.rsplit_once("HTTP status ")?;
    suffix
        .split_whitespace()
        .next()
        .and_then(|status| status.parse::<u16>().ok())
}

#[derive(Default)]
struct PendingMcpClientHttpSseEvent {
    event: Option<String>,
    id: Option<String>,
    retry_ms: Option<u64>,
    data_lines: Vec<String>,
    data_bytes: usize,
}

impl PendingMcpClientHttpSseEvent {
    fn push_data(&mut self, value: &str) -> Result<(), McpError> {
        let separator_bytes = usize::from(!self.data_lines.is_empty());
        self.data_bytes = self
            .data_bytes
            .checked_add(separator_bytes)
            .and_then(|bytes| bytes.checked_add(value.len()))
            .ok_or_else(mcp_client_http_sse_event_too_large)?;
        if self.data_bytes > MAX_CLIENT_HTTP_SSE_EVENT_DATA_BYTES {
            return Err(mcp_client_http_sse_event_too_large());
        }
        self.data_lines.push(value.to_string());
        Ok(())
    }

    fn reset(&mut self) {
        self.event = None;
        self.id = None;
        self.retry_ms = None;
        self.data_lines.clear();
        self.data_bytes = 0;
    }
}

fn flush_mcp_client_http_sse_event(
    pending: &mut PendingMcpClientHttpSseEvent,
    events: &mut Vec<McpClientHttpSseEvent>,
) -> Result<(), McpError> {
    if pending.data_lines.is_empty() {
        pending.reset();
        return Ok(());
    }

    let data = pending.data_lines.join("\n");
    if data.len() > MAX_CLIENT_HTTP_SSE_EVENT_DATA_BYTES {
        return Err(mcp_client_http_sse_event_too_large());
    }

    events.push(McpClientHttpSseEvent {
        event: pending.event.take(),
        id: pending.id.take(),
        retry_ms: pending.retry_ms.take(),
        data,
    });
    pending.reset();
    Ok(())
}

fn non_empty_sse_value(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn mcp_client_http_sse_event_too_large() -> McpError {
    McpError::InvalidServerRequest("MCP client HTTP SSE event data too large".to_string())
}
