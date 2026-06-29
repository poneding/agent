use std::collections::VecDeque;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{McpError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess};

const MAX_CLIENT_STDIO_FRAME_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientStdioResponseFrame {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}

#[async_trait]
pub trait McpClientStdioTransport: Send {
    async fn send_frame(&mut self, frame: String) -> Result<(), McpError>;

    async fn recv_frame(&mut self) -> Result<Option<String>, McpError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClientStdioTransport {
    inbound_frames: VecDeque<String>,
    sent_frames: Vec<String>,
}

impl FixtureMcpClientStdioTransport {
    #[must_use]
    pub fn from_inbound_frames<I, S>(frames: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            inbound_frames: frames.into_iter().map(Into::into).collect(),
            sent_frames: Vec::new(),
        }
    }

    #[must_use]
    pub fn sent_frames(&self) -> &[String] {
        &self.sent_frames
    }
}

#[async_trait]
impl McpClientStdioTransport for FixtureMcpClientStdioTransport {
    async fn send_frame(&mut self, frame: String) -> Result<(), McpError> {
        self.sent_frames.push(frame);
        Ok(())
    }

    async fn recv_frame(&mut self) -> Result<Option<String>, McpError> {
        Ok(self.inbound_frames.pop_front())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientStdioSessionStatus {
    Created,
    Initialized,
    Failed,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientStdioSessionSummary {
    pub status: McpClientStdioSessionStatus,
    pub requests_sent: u64,
    pub responses_received: u64,
    pub error_frames: u64,
    pub close_reason: Option<String>,
}

pub struct McpClientStdioSession<T> {
    transport: T,
    status: McpClientStdioSessionStatus,
    next_request_id: i64,
    requests_sent: u64,
    responses_received: u64,
    error_frames: u64,
    close_reason: Option<String>,
}

impl<T> McpClientStdioSession<T> {
    #[must_use]
    pub const fn new(transport: T) -> Self {
        Self {
            transport,
            status: McpClientStdioSessionStatus::Created,
            next_request_id: 1,
            requests_sent: 0,
            responses_received: 0,
            error_frames: 0,
            close_reason: None,
        }
    }

    #[must_use]
    pub const fn status(&self) -> McpClientStdioSessionStatus {
        self.status
    }

    #[must_use]
    pub const fn transport(&self) -> &T {
        &self.transport
    }

    #[must_use]
    pub const fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    #[must_use]
    pub fn summary(&self) -> McpClientStdioSessionSummary {
        McpClientStdioSessionSummary {
            status: self.status,
            requests_sent: self.requests_sent,
            responses_received: self.responses_received,
            error_frames: self.error_frames,
            close_reason: self.close_reason.clone(),
        }
    }

    pub fn mark_initialized(&mut self) {
        if self.status == McpClientStdioSessionStatus::Created {
            self.status = McpClientStdioSessionStatus::Initialized;
        }
    }

    pub fn close(&mut self, reason: impl Into<String>) -> McpClientStdioSessionSummary {
        if self.status != McpClientStdioSessionStatus::Closed {
            self.status = McpClientStdioSessionStatus::Closed;
            self.close_reason = Some(bound_text(&reason.into(), 256));
        }
        self.summary()
    }
}

impl<T> McpClientStdioSession<T>
where
    T: McpClientStdioTransport,
{
    pub async fn send_request(
        &mut self,
        method: impl Into<String>,
        params: Value,
    ) -> Result<McpClientStdioResponseFrame, McpError> {
        if self.status == McpClientStdioSessionStatus::Closed {
            return Err(self.record_failure("MCP client stdio session is closed"));
        }

        let id = self.allocate_request_id()?;
        let frame = render_mcp_client_stdio_request_frame(id.clone(), method, params)?;
        self.transport.send_frame(frame).await.inspect_err(|_| {
            self.status = McpClientStdioSessionStatus::Failed;
            self.error_frames += 1;
        })?;
        self.requests_sent += 1;

        let Some(response_frame) = self.transport.recv_frame().await.inspect_err(|_| {
            self.status = McpClientStdioSessionStatus::Failed;
            self.error_frames += 1;
        })?
        else {
            return Err(self.record_failure("MCP client stdio transport closed before response"));
        };

        self.responses_received += 1;
        let response =
            parse_mcp_client_stdio_response_frame(&response_frame).inspect_err(|_| {
                self.status = McpClientStdioSessionStatus::Failed;
                self.error_frames += 1;
            })?;

        if response.id() != &id {
            return Err(self.record_failure("MCP client stdio unexpected response id"));
        }
        if matches!(response, McpClientStdioResponseFrame::Failure(_)) {
            self.error_frames += 1;
        }

        Ok(response)
    }

    fn allocate_request_id(&mut self) -> Result<McpJsonRpcId, McpError> {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.checked_add(1).ok_or_else(|| {
            McpError::InvalidServerRequest("MCP client stdio request id exhausted".to_string())
        })?;
        Ok(McpJsonRpcId::Number(id))
    }

    fn record_failure(&mut self, message: &str) -> McpError {
        self.status = McpClientStdioSessionStatus::Failed;
        self.error_frames += 1;
        McpError::InvalidServerRequest(message.to_string())
    }
}

impl McpClientStdioResponseFrame {
    #[must_use]
    pub const fn id(&self) -> &McpJsonRpcId {
        match self {
            Self::Success(success) => &success.id,
            Self::Failure(failure) => &failure.id,
        }
    }
}

pub fn render_mcp_client_stdio_request_frame(
    id: McpJsonRpcId,
    method: impl Into<String>,
    params: Value,
) -> Result<String, McpError> {
    if matches!(id, McpJsonRpcId::Null) {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio request id cannot be null".to_string(),
        ));
    }

    let method = method.into();
    if method.trim().is_empty() {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio method cannot be empty".to_string(),
        ));
    }

    let request = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let mut rendered = serde_json::to_string(&request).map_err(|_| {
        McpError::InvalidServerRequest("failed to serialize MCP client stdio request".to_string())
    })?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn parse_mcp_client_stdio_response_frame(
    frame: &str,
) -> Result<McpClientStdioResponseFrame, McpError> {
    if frame.len() > MAX_CLIENT_STDIO_FRAME_BYTES {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio response frame too large".to_string(),
        ));
    }

    let value = serde_json::from_str::<Value>(frame.trim_end()).map_err(|_| {
        McpError::InvalidServerRequest("invalid MCP client stdio response frame".to_string())
    })?;
    if value.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio response JSON-RPC version must be 2.0".to_string(),
        ));
    }
    if value.get("id").is_none() {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio response id missing".to_string(),
        ));
    }

    match (value.get("result").is_some(), value.get("error").is_some()) {
        (true, false) => serde_json::from_value::<McpJsonRpcSuccess>(value)
            .map(McpClientStdioResponseFrame::Success)
            .map_err(|_| {
                McpError::InvalidServerRequest(
                    "invalid MCP client stdio success response".to_string(),
                )
            }),
        (false, true) => serde_json::from_value::<McpJsonRpcFailure>(value)
            .map(McpClientStdioResponseFrame::Failure)
            .map_err(|_| {
                McpError::InvalidServerRequest(
                    "invalid MCP client stdio failure response".to_string(),
                )
            }),
        _ => Err(McpError::InvalidServerRequest(
            "MCP client stdio response must contain exactly one result or error".to_string(),
        )),
    }
}

fn bound_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}
