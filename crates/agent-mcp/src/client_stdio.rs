use std::collections::VecDeque;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::{
    MCP_SERVER_PROTOCOL_VERSION, McpClient, McpEnvValue, McpError, McpInitializeResult,
    McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess, McpServerCapabilities, McpServerConfig,
    McpServerId, McpServerMethod, McpToolDescriptor, McpToolListResult, McpTransportConfig,
};

const MAX_CLIENT_STDIO_FRAME_BYTES: usize = 64 * 1024;
const CLIENT_STDIO_CLOSE_TIMEOUT: Duration = Duration::from_millis(1_000);
const CLIENT_STDIO_EXIT_POLL_TIMEOUT: Duration = Duration::from_millis(100);

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

pub struct McpClientStdioProcessTransport {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    timeout: Duration,
}

impl McpClientStdioProcessTransport {
    pub async fn spawn(config: &McpServerConfig) -> Result<Self, McpError> {
        if !config.enabled {
            return Err(McpError::InvalidConfig(
                "MCP client stdio server is disabled".to_string(),
            ));
        }

        let McpTransportConfig::Stdio { command, args, env } = &config.transport else {
            return Err(McpError::InvalidConfig(
                "MCP client stdio transport requires stdio config".to_string(),
            ));
        };

        let mut process = Command::new(command);
        process
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        for (key, value) in env {
            match value {
                McpEnvValue::Literal(value) => {
                    process.env(key, value);
                }
                McpEnvValue::SecretRef(_) => {
                    return Err(McpError::InvalidConfig(
                        "MCP client stdio env contains unresolved secret ref".to_string(),
                    ));
                }
            }
        }

        let mut child = process.spawn().map_err(|_| {
            McpError::Client("failed to spawn MCP client stdio process".to_string())
        })?;

        let Some(stdin) = child.stdin.take() else {
            let _ = child.start_kill();
            return Err(McpError::Client(
                "MCP client stdio process stdin unavailable".to_string(),
            ));
        };
        let Some(stdout) = child.stdout.take() else {
            let _ = child.start_kill();
            return Err(McpError::Client(
                "MCP client stdio process stdout unavailable".to_string(),
            ));
        };

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            timeout: Duration::from_millis(config.timeout_ms),
        })
    }

    pub async fn close(&mut self) -> Result<(), McpError> {
        let _ = self.stdin.shutdown().await;
        if self
            .child
            .try_wait()
            .map_err(|_| mcp_client_stdio_process_state_error())?
            .is_some()
        {
            return Ok(());
        }

        self.terminate_child();
        match timeout(CLIENT_STDIO_CLOSE_TIMEOUT, self.child.wait()).await {
            Ok(Ok(_)) | Err(_) => Ok(()),
            Ok(Err(_)) => Err(McpError::Client(
                "failed to close MCP client stdio process".to_string(),
            )),
        }
    }

    fn terminate_child(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[async_trait]
impl McpClientStdioTransport for McpClientStdioProcessTransport {
    async fn send_frame(&mut self, frame: String) -> Result<(), McpError> {
        match timeout(self.timeout, async {
            self.stdin
                .write_all(frame.as_bytes())
                .await
                .map_err(|_| mcp_client_stdio_write_error())?;
            self.stdin
                .flush()
                .await
                .map_err(|_| mcp_client_stdio_write_error())
        })
        .await
        {
            Ok(result) => result,
            Err(_) => {
                self.terminate_child();
                Err(McpError::Client(
                    "MCP client stdio write timed out".to_string(),
                ))
            }
        }
    }

    async fn recv_frame(&mut self) -> Result<Option<String>, McpError> {
        let mut frame = String::new();
        let bytes_read = match timeout(self.timeout, self.stdout.read_line(&mut frame)).await {
            Ok(Ok(bytes_read)) => bytes_read,
            Ok(Err(_)) => return Err(McpError::Client("MCP client stdio read failed".to_string())),
            Err(_) => {
                self.terminate_child();
                return Err(McpError::Client(
                    "MCP client stdio read timed out".to_string(),
                ));
            }
        };

        if bytes_read == 0 {
            return match self
                .child
                .try_wait()
                .map_err(|_| mcp_client_stdio_process_state_error())?
            {
                Some(status) => Err(mcp_client_stdio_process_exit_error(status)),
                None => match timeout(CLIENT_STDIO_EXIT_POLL_TIMEOUT, self.child.wait()).await {
                    Ok(Ok(status)) => Err(mcp_client_stdio_process_exit_error(status)),
                    Ok(Err(_)) => Err(mcp_client_stdio_process_state_error()),
                    Err(_) => Ok(None),
                },
            };
        }

        if frame.len() > MAX_CLIENT_STDIO_FRAME_BYTES {
            return Err(McpError::InvalidServerRequest(
                "MCP client stdio response frame too large".to_string(),
            ));
        }

        Ok(Some(frame))
    }
}

impl Drop for McpClientStdioProcessTransport {
    fn drop(&mut self) {
        self.terminate_child();
    }
}

pub struct McpStdioClient<T> {
    state: Mutex<McpStdioClientState<T>>,
}

struct McpStdioClientState<T> {
    session: McpClientStdioSession<T>,
    server_id: Option<McpServerId>,
}

impl<T> McpStdioClient<T> {
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self::from_session(McpClientStdioSession::new(transport))
    }

    #[must_use]
    pub fn from_session(session: McpClientStdioSession<T>) -> Self {
        Self {
            state: Mutex::new(McpStdioClientState {
                session,
                server_id: None,
            }),
        }
    }
}

impl McpStdioClient<McpClientStdioProcessTransport> {
    pub async fn spawn(config: &McpServerConfig) -> Result<Self, McpError> {
        Ok(Self::new(
            McpClientStdioProcessTransport::spawn(config).await?,
        ))
    }

    pub async fn close(&self) -> Result<(), McpError> {
        self.state
            .lock()
            .await
            .session
            .transport_mut()
            .close()
            .await
    }
}

#[async_trait]
impl<T> McpClient for McpStdioClient<T>
where
    T: McpClientStdioTransport,
{
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError> {
        let mut state = self.state.lock().await;
        let response = state
            .session
            .send_request(
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
        let result = mcp_client_stdio_success_result(response, "initialize")?;
        let initialize = map_mcp_client_stdio_initialize_result(config.id.clone(), &result)?;
        state.session.mark_initialized();
        state.server_id = Some(config.id.clone());
        Ok(initialize)
    }

    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError> {
        let mut state = self.state.lock().await;
        if state.server_id.as_ref() != Some(server_id) {
            return Err(McpError::Client(
                "MCP client stdio session is not initialized".to_string(),
            ));
        }

        let response = state
            .session
            .send_request(McpServerMethod::ToolsList.as_str(), json!({}))
            .await?;
        let result = mcp_client_stdio_success_result(response, "tools/list")?;
        map_mcp_client_stdio_tool_list_result(server_id.clone(), &result)
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

fn mcp_client_stdio_write_error() -> McpError {
    McpError::Client("MCP client stdio write failed".to_string())
}

fn mcp_client_stdio_process_state_error() -> McpError {
    McpError::Client("MCP client stdio process state unavailable".to_string())
}

fn mcp_client_stdio_process_exit_error(status: ExitStatus) -> McpError {
    let message = status.code().map_or_else(
        || "MCP client stdio process exited before response".to_string(),
        |code| format!("MCP client stdio process exited before response with status {code}"),
    );
    McpError::Client(message)
}

fn mcp_client_stdio_success_result(
    response: McpClientStdioResponseFrame,
    operation: &str,
) -> Result<Value, McpError> {
    match response {
        McpClientStdioResponseFrame::Success(success) => Ok(success.result),
        McpClientStdioResponseFrame::Failure(failure) => Err(McpError::Client(format!(
            "MCP client stdio {operation} failed with code {}",
            failure.error.code
        ))),
    }
}

fn map_mcp_client_stdio_initialize_result(
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

fn map_mcp_client_stdio_tool_list_result(
    server_id: McpServerId,
    result: &Value,
) -> Result<McpToolListResult, McpError> {
    let tools = result
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_mcp_client_stdio_result("tools/list"))?;
    let mut descriptors = Vec::with_capacity(tools.len());
    for tool in tools {
        let tool = tool
            .as_object()
            .ok_or_else(|| invalid_mcp_client_stdio_result("tools/list"))?;
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

fn object_field<'a>(
    value: &'a Value,
    field: &str,
    operation: &str,
) -> Result<&'a serde_json::Map<String, Value>, McpError> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_mcp_client_stdio_result(operation))
}

fn object_field_any<'a>(
    value: &'a Value,
    fields: &[&str],
    operation: &str,
) -> Result<&'a serde_json::Map<String, Value>, McpError> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(Value::as_object))
        .ok_or_else(|| invalid_mcp_client_stdio_result(operation))
}

fn value_field_any<'a>(
    value: &'a serde_json::Map<String, Value>,
    fields: &[&str],
    operation: &str,
) -> Result<&'a Value, McpError> {
    fields
        .iter()
        .find_map(|field| value.get(*field))
        .ok_or_else(|| invalid_mcp_client_stdio_result(operation))
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
        .ok_or_else(|| invalid_mcp_client_stdio_result(operation))
}

fn string_field_any(value: &Value, fields: &[&str], operation: &str) -> Result<String, McpError> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(Value::as_str))
        .filter(|text| !text.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| invalid_mcp_client_stdio_result(operation))
}

fn optional_string_field(
    value: &serde_json::Map<String, Value>,
    field: &str,
    operation: &str,
) -> Result<Option<String>, McpError> {
    match value.get(field) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text.clone())),
        Some(Value::String(_)) | Some(Value::Null) | None => Ok(None),
        Some(_) => Err(invalid_mcp_client_stdio_result(operation)),
    }
}

fn capability_enabled(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(enabled)) => *enabled,
        Some(Value::Object(_)) => true,
        _ => false,
    }
}

fn invalid_mcp_client_stdio_result(operation: &str) -> McpError {
    McpError::Client(format!("invalid MCP client stdio {operation} result"))
}
