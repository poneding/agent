use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId, McpServerFixture, handle_mcp_server_frame,
};

const DEFAULT_SESSION_MAX_FRAME_BYTES: usize = 64 * 1024;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerSessionStatus {
    Created,
    Initialized,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerSessionConfig {
    pub max_frame_bytes: usize,
    pub max_frames: Option<u64>,
}

impl Default for McpServerSessionConfig {
    fn default() -> Self {
        Self {
            max_frame_bytes: DEFAULT_SESSION_MAX_FRAME_BYTES,
            max_frames: None,
        }
    }
}

impl McpServerSessionConfig {
    #[must_use]
    pub const fn new(max_frame_bytes: usize, max_frames: Option<u64>) -> Self {
        Self {
            max_frame_bytes,
            max_frames,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerSessionSummary {
    pub status: McpServerSessionStatus,
    pub frames_received: u64,
    pub responses_sent: u64,
    pub error_frames: u64,
    pub close_reason: Option<String>,
}

pub struct McpServerSession {
    fixture: McpServerFixture,
    config: McpServerSessionConfig,
    status: McpServerSessionStatus,
    frames_received: u64,
    responses_sent: u64,
    error_frames: u64,
    close_reason: Option<String>,
}

impl McpServerSession {
    #[must_use]
    pub const fn new(fixture: McpServerFixture, config: McpServerSessionConfig) -> Self {
        Self {
            fixture,
            config,
            status: McpServerSessionStatus::Created,
            frames_received: 0,
            responses_sent: 0,
            error_frames: 0,
            close_reason: None,
        }
    }

    #[must_use]
    pub const fn status(&self) -> McpServerSessionStatus {
        self.status
    }

    #[must_use]
    pub fn summary(&self) -> McpServerSessionSummary {
        McpServerSessionSummary {
            status: self.status,
            frames_received: self.frames_received,
            responses_sent: self.responses_sent,
            error_frames: self.error_frames,
            close_reason: self.close_reason.clone(),
        }
    }

    pub fn handle_frame(&mut self, frame: &str) -> String {
        if self.status == McpServerSessionStatus::Closed {
            return self.record_error_response("MCP server session is closed");
        }
        if self
            .config
            .max_frames
            .is_some_and(|max_frames| self.frames_received >= max_frames)
        {
            self.close("frame limit reached");
            return self.record_error_response("MCP server session frame limit reached");
        }

        self.frames_received += 1;
        if frame.len() > self.config.max_frame_bytes {
            return self.record_error_response("JSON-RPC frame too large");
        }

        let response = handle_mcp_server_frame(&mut self.fixture, frame);
        if self.fixture.state().initialized {
            self.status = McpServerSessionStatus::Initialized;
        }
        self.record_response(&response);

        if self
            .config
            .max_frames
            .is_some_and(|max_frames| self.frames_received >= max_frames)
        {
            self.close("frame limit reached");
        }

        response
    }

    pub fn close(&mut self, reason: impl Into<String>) -> McpServerSessionSummary {
        self.status = McpServerSessionStatus::Closed;
        self.close_reason = Some(bound_text(&reason.into(), 256));
        self.summary()
    }

    fn record_response(&mut self, response: &str) {
        if response.is_empty() {
            return;
        }
        self.responses_sent += 1;
        if response_is_error(response) {
            self.error_frames += 1;
        }
    }

    fn record_error_response(&mut self, message: &str) -> String {
        let response = render_failure(message);
        self.record_response(&response);
        response
    }
}

fn response_is_error(response: &str) -> bool {
    serde_json::from_str::<Value>(response.trim_end())
        .ok()
        .and_then(|value| value.get("error").cloned())
        .is_some()
}

fn render_failure(message: &str) -> String {
    let failure = McpJsonRpcFailure::new(
        McpJsonRpcId::Null,
        McpJsonRpcError::invalid_request(message),
    );
    let mut rendered = serde_json::to_string(&failure).expect("JSON-RPC failures are serializable");
    rendered.push('\n');
    rendered
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
