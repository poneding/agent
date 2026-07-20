use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_core::Stream;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::config::{ModelProviderConfig, ModelProviderKind, SecretRef};
use crate::provider::{ModelError, ModelProvider};
use crate::request::{ModelMessage, ModelRequest};
use crate::secrets::SecretResolver;
use crate::stream::{ModelEventStream, ModelStreamEvent, ModelStreamResult};

const DEFAULT_OPENAI_COMPATIBLE_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiCompatibleHttpRequest {
    pub url: String,
    pub secret_ref: SecretRef,
    pub body: Value,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiCompatibleHttpResponse {
    pub request_id: String,
    pub sse_chunks: Vec<String>,
}

pub type OpenAiCompatibleChunkStream =
    Pin<Box<dyn Stream<Item = Result<String, OpenAiCompatibleError>> + Send>>;

pub struct OpenAiCompatibleHttpStreamResponse {
    pub request_id: String,
    pub sse_chunks: OpenAiCompatibleChunkStream,
}

impl OpenAiCompatibleHttpStreamResponse {
    #[must_use]
    pub fn from_response(response: OpenAiCompatibleHttpResponse) -> Self {
        Self {
            request_id: response.request_id,
            sse_chunks: Box::pin(stream::iter(response.sse_chunks.into_iter().map(Ok))),
        }
    }
}

#[async_trait]
pub trait OpenAiCompatibleTransport: Send + Sync {
    async fn send(
        &self,
        request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError>;

    async fn send_stream(
        &self,
        request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpStreamResponse, OpenAiCompatibleError> {
        self.send(request)
            .await
            .map(OpenAiCompatibleHttpStreamResponse::from_response)
    }
}

#[derive(Clone, Debug)]
pub struct FixtureOpenAiCompatibleTransport {
    response: Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError>,
}

impl FixtureOpenAiCompatibleTransport {
    #[must_use]
    pub fn with_chunks(request_id: impl Into<String>, sse_chunks: Vec<String>) -> Self {
        Self {
            response: Ok(OpenAiCompatibleHttpResponse {
                request_id: request_id.into(),
                sse_chunks,
            }),
        }
    }

    #[must_use]
    pub fn with_error(error: OpenAiCompatibleError) -> Self {
        Self {
            response: Err(error),
        }
    }
}

#[async_trait]
impl OpenAiCompatibleTransport for FixtureOpenAiCompatibleTransport {
    async fn send(
        &self,
        _request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError> {
        self.response.clone()
    }
}

#[derive(Clone)]
pub struct ReqwestOpenAiCompatibleTransport {
    client: reqwest::Client,
    secret_resolver: Arc<dyn SecretResolver>,
}

impl ReqwestOpenAiCompatibleTransport {
    #[must_use]
    pub fn new(secret_resolver: Arc<dyn SecretResolver>) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret_resolver,
        }
    }

    #[must_use]
    pub fn with_client(secret_resolver: Arc<dyn SecretResolver>, client: reqwest::Client) -> Self {
        Self {
            client,
            secret_resolver,
        }
    }
}

impl fmt::Debug for ReqwestOpenAiCompatibleTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReqwestOpenAiCompatibleTransport")
            .field("client", &"<reqwest client>")
            .field("secret_resolver", &"<redacted>")
            .finish()
    }
}

#[async_trait]
impl OpenAiCompatibleTransport for ReqwestOpenAiCompatibleTransport {
    async fn send(
        &self,
        request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpResponse, OpenAiCompatibleError> {
        let response = self.send_stream(request).await?;
        let request_id = response.request_id;
        let mut sse_chunks = Vec::new();
        let mut chunks = response.sse_chunks;
        while let Some(chunk) = chunks.next().await {
            sse_chunks.push(chunk?);
        }

        Ok(OpenAiCompatibleHttpResponse {
            request_id,
            sse_chunks,
        })
    }

    async fn send_stream(
        &self,
        request: OpenAiCompatibleHttpRequest,
    ) -> Result<OpenAiCompatibleHttpStreamResponse, OpenAiCompatibleError> {
        let secret = self
            .secret_resolver
            .resolve(&request.secret_ref)
            .map_err(|error| OpenAiCompatibleError::SecretResolution {
                message: error.to_string(),
            })?;
        let response = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .bearer_auth(secret.expose_secret())
            .json(&request.body)
            .send()
            .await
            .map_err(|error| OpenAiCompatibleError::Transport {
                message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
            })?;

        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .or_else(|| response.headers().get("openai-request-id"))
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("reqwest-response")
            .to_string();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(OpenAiCompatibleError::provider_response(
                status.as_u16(),
                &body,
            ));
        }

        let sse_chunks = response.bytes_stream().map(|chunk| {
            chunk
                .map(|chunk| String::from_utf8_lossy(&chunk).to_string())
                .map_err(|error| OpenAiCompatibleError::Transport {
                    message: redact_secret_like(&sanitize_url_in_message(&error.to_string())),
                })
        });

        Ok(OpenAiCompatibleHttpStreamResponse {
            request_id,
            sse_chunks: Box::pin(sse_chunks),
        })
    }
}

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    config: ModelProviderConfig,
    secret_ref: SecretRef,
    transport: Arc<dyn OpenAiCompatibleTransport>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        config: ModelProviderConfig,
        transport: Arc<dyn OpenAiCompatibleTransport>,
    ) -> Result<Self, OpenAiCompatibleError> {
        if config.kind != ModelProviderKind::OpenAiCompatible {
            return Err(OpenAiCompatibleError::InvalidProviderKind {
                provider_id: config.id,
            });
        }
        let Some(secret_ref) = config.api_key.clone() else {
            return Err(OpenAiCompatibleError::MissingSecretRef {
                provider_id: config.id,
            });
        };
        config
            .validate()
            .map_err(|err| OpenAiCompatibleError::InvalidConfig {
                message: err.to_string(),
            })?;
        Ok(Self {
            config,
            secret_ref,
            transport,
        })
    }

    #[must_use]
    pub fn config(&self) -> &ModelProviderConfig {
        &self.config
    }
}

impl fmt::Debug for OpenAiCompatibleProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAiCompatibleProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl ModelProvider for OpenAiCompatibleProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        let responses_request = OpenAiCompatibleHttpRequest {
            url: openai_compatible_responses_url(&self.config),
            secret_ref: self.secret_ref.clone(),
            body: build_openai_compatible_request_body(&request),
            timeout_ms: self.config.endpoint.timeout_ms,
        };
        let response = match self.transport.send_stream(responses_request).await {
            Ok(response) => response,
            Err(error) if should_try_chat_completions_fallback(&error) => {
                let chat_request = OpenAiCompatibleHttpRequest {
                    url: openai_compatible_chat_completions_url(&self.config),
                    secret_ref: self.secret_ref.clone(),
                    body: build_openai_compatible_chat_request_body(&request),
                    timeout_ms: self.config.endpoint.timeout_ms,
                };
                let chat_response = self.transport.send_stream(chat_request).await?;
                return Ok(openai_compatible_chat_stream(chat_response));
            }
            Err(error) => return Err(error.into()),
        };
        Ok(openai_compatible_response_stream(response))
    }
}

#[must_use]
pub fn build_openai_compatible_request_body(request: &ModelRequest) -> Value {
    let input = request
        .messages
        .iter()
        .map(openai_responses_message)
        .collect::<Vec<_>>();

    let mut body = json!({
        "model": request.model,
        "input": input,
        "stream": true,
    });
    if let Some(effort) = request.reasoning_effort {
        body["reasoning"] = json!({ "effort": effort.label() });
    }
    if !request.tools.is_empty() {
        body["tools"] = json!(
            request
                .tools
                .iter()
                .map(|tool| json!({
                    "type": "function",
                    "name": openai_compatible_tool_name(&tool.name),
                    "description": tool.description.clone(),
                    "parameters": tool.input_schema.clone(),
                }))
                .collect::<Vec<_>>()
        );
    }
    body
}

#[must_use]
pub fn build_openai_compatible_chat_request_body(request: &ModelRequest) -> Value {
    let messages = request
        .messages
        .iter()
        .map(openai_chat_message)
        .collect::<Vec<_>>();

    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
    });
    if let Some(effort) = request.reasoning_effort {
        body["reasoning_effort"] = json!(effort.label());
    }
    if !request.tools.is_empty() {
        body["tools"] = json!(
            request
                .tools
                .iter()
                .map(|tool| json!({
                    "type": "function",
                    "function": {
                        "name": openai_compatible_tool_name(&tool.name),
                        "description": tool.description.clone(),
                        "parameters": tool.input_schema.clone(),
                    }
                }))
                .collect::<Vec<_>>()
        );
    }
    body
}

fn openai_responses_message(message: &ModelMessage) -> Value {
    match message {
        ModelMessage::System { content } => json!({
            "role": "system",
            "content": content,
        }),
        ModelMessage::User { content } => json!({
            "role": "user",
            "content": content,
        }),
        ModelMessage::Assistant { content } => json!({
            "role": "assistant",
            "content": content,
        }),
        ModelMessage::AssistantToolCall {
            call_id,
            tool_name,
            input_json,
        } => json!({
            "type": "function_call",
            "call_id": call_id,
            "name": openai_compatible_tool_name(tool_name),
            "arguments": input_json.to_string(),
        }),
        ModelMessage::ToolResult {
            call_id, content, ..
        } => json!({
            "type": "function_call_output",
            "call_id": call_id,
            "output": content,
        }),
    }
}

fn openai_chat_message(message: &ModelMessage) -> Value {
    match message {
        ModelMessage::System { content } => json!({
            "role": "system",
            "content": content,
        }),
        ModelMessage::User { content } => json!({
            "role": "user",
            "content": content,
        }),
        ModelMessage::Assistant { content } => json!({
            "role": "assistant",
            "content": content,
        }),
        ModelMessage::AssistantToolCall {
            call_id,
            tool_name,
            input_json,
        } => json!({
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": call_id,
                "type": "function",
                "function": {
                    "name": openai_compatible_tool_name(tool_name),
                    "arguments": input_json.to_string(),
                }
            }],
        }),
        ModelMessage::ToolResult {
            call_id,
            tool_name,
            content,
        } => json!({
            "role": "tool",
            "tool_call_id": call_id,
            "name": openai_compatible_tool_name(tool_name),
            "content": content,
        }),
    }
}

pub fn parse_openai_compatible_sse(
    request_id: &str,
    response: &OpenAiCompatibleHttpResponse,
) -> Result<ModelStreamResult, OpenAiCompatibleError> {
    let mut events = vec![ModelStreamEvent::Started {
        request_id: request_id.to_string(),
    }];
    let mut buffer = String::new();
    let mut tool_calls = BTreeMap::new();

    for chunk in &response.sse_chunks {
        buffer.push_str(chunk);
        while let Some((frame_end, drain_to)) = next_sse_frame_boundary(&buffer) {
            let frame = buffer[..frame_end].trim().to_string();
            buffer.drain(..drain_to);
            handle_openai_compatible_sse_frame(&frame, &mut events, &mut tool_calls)?;
        }
    }

    let trailing = buffer.trim();
    if !trailing.is_empty() {
        match handle_openai_compatible_sse_frame(trailing, &mut events, &mut tool_calls) {
            Ok(()) => {}
            Err(error) if is_incomplete_json_stream_event(&error) => {}
            Err(error) => return Err(error),
        }
    }
    flush_openai_response_tool_calls(&mut tool_calls, &mut events)?;

    Ok(events)
}

pub fn parse_openai_compatible_chat_sse(
    request_id: &str,
    response: &OpenAiCompatibleHttpResponse,
) -> Result<ModelStreamResult, OpenAiCompatibleError> {
    let mut events = vec![ModelStreamEvent::Started {
        request_id: request_id.to_string(),
    }];
    let mut buffer = String::new();
    let mut tool_calls = BTreeMap::new();

    for chunk in &response.sse_chunks {
        buffer.push_str(chunk);
        while let Some((frame_end, drain_to)) = next_sse_frame_boundary(&buffer) {
            let frame = buffer[..frame_end].trim().to_string();
            buffer.drain(..drain_to);
            handle_openai_compatible_chat_sse_frame(&frame, &mut events, &mut tool_calls)?;
        }
    }

    let trailing = buffer.trim();
    if !trailing.is_empty() {
        match handle_openai_compatible_chat_sse_frame(trailing, &mut events, &mut tool_calls) {
            Ok(()) => {}
            Err(error) if is_incomplete_json_stream_event(&error) => {}
            Err(error) => return Err(error),
        }
    }
    flush_openai_chat_tool_calls(&mut tool_calls, &mut events)?;

    Ok(events)
}

fn openai_compatible_response_stream(
    response: OpenAiCompatibleHttpStreamResponse,
) -> ModelEventStream {
    let state = OpenAiResponseStreamState {
        request_id: response.request_id,
        sse_chunks: response.sse_chunks,
        buffer: String::new(),
        tool_calls: BTreeMap::new(),
        pending: VecDeque::new(),
        started: false,
        done: false,
    };
    Box::pin(stream::unfold(state, next_openai_response_stream_item))
}

fn openai_compatible_chat_stream(response: OpenAiCompatibleHttpStreamResponse) -> ModelEventStream {
    let state = OpenAiChatStreamState {
        request_id: response.request_id,
        sse_chunks: response.sse_chunks,
        buffer: String::new(),
        tool_calls: BTreeMap::new(),
        pending: VecDeque::new(),
        started: false,
        done: false,
    };
    Box::pin(stream::unfold(state, next_openai_chat_stream_item))
}

struct OpenAiResponseStreamState {
    request_id: String,
    sse_chunks: OpenAiCompatibleChunkStream,
    buffer: String,
    tool_calls: BTreeMap<String, OpenAiResponseToolCallState>,
    pending: VecDeque<ModelStreamEvent>,
    started: bool,
    done: bool,
}

struct OpenAiChatStreamState {
    request_id: String,
    sse_chunks: OpenAiCompatibleChunkStream,
    buffer: String,
    tool_calls: BTreeMap<u64, OpenAiChatToolCallState>,
    pending: VecDeque<ModelStreamEvent>,
    started: bool,
    done: bool,
}

async fn next_openai_response_stream_item(
    mut state: OpenAiResponseStreamState,
) -> Option<(
    Result<ModelStreamEvent, ModelError>,
    OpenAiResponseStreamState,
)> {
    if !state.started {
        state.started = true;
        return Some((
            Ok(ModelStreamEvent::Started {
                request_id: state.request_id.clone(),
            }),
            state,
        ));
    }

    loop {
        if let Some(event) = state.pending.pop_front() {
            return Some((Ok(event), state));
        }
        if state.done {
            return None;
        }

        match state.sse_chunks.next().await {
            Some(Ok(chunk)) => {
                if let Err(error) = push_openai_response_chunk(&mut state, &chunk) {
                    state.done = true;
                    return Some((Err(error.into()), state));
                }
            }
            Some(Err(error)) => {
                state.done = true;
                return Some((Err(error.into()), state));
            }
            None => {
                if let Err(error) = finish_openai_response_stream(&mut state) {
                    state.done = true;
                    return Some((Err(error.into()), state));
                }
                state.done = true;
            }
        }
    }
}

async fn next_openai_chat_stream_item(
    mut state: OpenAiChatStreamState,
) -> Option<(Result<ModelStreamEvent, ModelError>, OpenAiChatStreamState)> {
    if !state.started {
        state.started = true;
        return Some((
            Ok(ModelStreamEvent::Started {
                request_id: state.request_id.clone(),
            }),
            state,
        ));
    }

    loop {
        if let Some(event) = state.pending.pop_front() {
            return Some((Ok(event), state));
        }
        if state.done {
            return None;
        }

        match state.sse_chunks.next().await {
            Some(Ok(chunk)) => {
                if let Err(error) = push_openai_chat_chunk(&mut state, &chunk) {
                    state.done = true;
                    return Some((Err(error.into()), state));
                }
            }
            Some(Err(error)) => {
                state.done = true;
                return Some((Err(error.into()), state));
            }
            None => {
                if let Err(error) = finish_openai_chat_stream(&mut state) {
                    state.done = true;
                    return Some((Err(error.into()), state));
                }
                state.done = true;
            }
        }
    }
}

fn push_openai_response_chunk(
    state: &mut OpenAiResponseStreamState,
    chunk: &str,
) -> Result<(), OpenAiCompatibleError> {
    state.buffer.push_str(chunk);
    while let Some((frame_end, drain_to)) = next_sse_frame_boundary(&state.buffer) {
        let frame = state.buffer[..frame_end].trim().to_string();
        state.buffer.drain(..drain_to);
        let mut events = Vec::new();
        handle_openai_compatible_sse_frame(&frame, &mut events, &mut state.tool_calls)?;
        state.pending.extend(events);
    }
    Ok(())
}

fn push_openai_chat_chunk(
    state: &mut OpenAiChatStreamState,
    chunk: &str,
) -> Result<(), OpenAiCompatibleError> {
    state.buffer.push_str(chunk);
    while let Some((frame_end, drain_to)) = next_sse_frame_boundary(&state.buffer) {
        let frame = state.buffer[..frame_end].trim().to_string();
        state.buffer.drain(..drain_to);
        let mut events = Vec::new();
        handle_openai_compatible_chat_sse_frame(&frame, &mut events, &mut state.tool_calls)?;
        state.pending.extend(events);
    }
    Ok(())
}

fn finish_openai_response_stream(
    state: &mut OpenAiResponseStreamState,
) -> Result<(), OpenAiCompatibleError> {
    let trailing = state.buffer.trim().to_string();
    state.buffer.clear();
    if !trailing.is_empty() {
        let mut events = Vec::new();
        match handle_openai_compatible_sse_frame(&trailing, &mut events, &mut state.tool_calls) {
            Ok(()) => state.pending.extend(events),
            Err(error) if is_incomplete_json_stream_event(&error) => {}
            Err(error) => return Err(error),
        }
    }
    let mut events = Vec::new();
    flush_openai_response_tool_calls(&mut state.tool_calls, &mut events)?;
    state.pending.extend(events);
    Ok(())
}

fn finish_openai_chat_stream(
    state: &mut OpenAiChatStreamState,
) -> Result<(), OpenAiCompatibleError> {
    let trailing = state.buffer.trim().to_string();
    state.buffer.clear();
    if !trailing.is_empty() {
        let mut events = Vec::new();
        match handle_openai_compatible_chat_sse_frame(&trailing, &mut events, &mut state.tool_calls)
        {
            Ok(()) => state.pending.extend(events),
            Err(error) if is_incomplete_json_stream_event(&error) => {}
            Err(error) => return Err(error),
        }
    }
    let mut events = Vec::new();
    flush_openai_chat_tool_calls(&mut state.tool_calls, &mut events)?;
    state.pending.extend(events);
    Ok(())
}

fn handle_openai_compatible_sse_frame(
    frame: &str,
    events: &mut ModelStreamResult,
    tool_calls: &mut BTreeMap<String, OpenAiResponseToolCallState>,
) -> Result<(), OpenAiCompatibleError> {
    if frame.is_empty() {
        return Ok(());
    }
    let Some((event_name, data)) = parse_sse_frame(frame) else {
        return Ok(());
    };
    if data == "[DONE]" {
        return Ok(());
    }
    let value: Value =
        serde_json::from_str(&data).map_err(|err| OpenAiCompatibleError::InvalidStreamEvent {
            message: format!("invalid JSON stream data: {err}"),
        })?;

    match event_name.as_deref() {
        Some("response.reasoning_summary_text.delta")
        | Some("response.reasoning_summary.delta")
        | Some("response.reasoning.delta") => {
            if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                events.push(ModelStreamEvent::ReasoningSummaryDelta {
                    text: delta.to_string(),
                });
            }
        }
        Some("response.output_text.delta") => {
            if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                events.push(ModelStreamEvent::Delta {
                    text: delta.to_string(),
                });
            }
        }
        Some("response.output_item.added") => {
            if let Some(item) = value.get("item") {
                record_openai_response_tool_item(item, tool_calls);
            }
        }
        Some("response.function_call_arguments.delta") => {
            let key = response_tool_call_key(&value);
            if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                tool_calls.entry(key).or_default().arguments.push_str(delta);
            }
        }
        Some("response.output_item.done") => {
            if let Some(item) = value.get("item") {
                if let Some(event) = openai_response_tool_event_from_item(item)? {
                    events.push(event);
                    if let Some(key) = item.get("id").and_then(Value::as_str) {
                        tool_calls.remove(key);
                    }
                }
            }
        }
        Some("response.completed") => {
            flush_openai_response_tool_calls(tool_calls, events)?;
            events.push(ModelStreamEvent::Completed {
                finish_reason: value
                    .get("finish_reason")
                    .and_then(Value::as_str)
                    .unwrap_or("stop")
                    .to_string(),
                output_token_count: value
                    .get("output_token_count")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
                    .min(u64::from(u32::MAX)) as u32,
            });
        }
        _ => {}
    }

    Ok(())
}

fn handle_openai_compatible_chat_sse_frame(
    frame: &str,
    events: &mut ModelStreamResult,
    tool_calls: &mut BTreeMap<u64, OpenAiChatToolCallState>,
) -> Result<(), OpenAiCompatibleError> {
    if frame.is_empty() {
        return Ok(());
    }
    let Some((_event_name, data)) = parse_sse_frame(frame) else {
        return Ok(());
    };
    if data == "[DONE]" {
        return Ok(());
    }
    let value: Value =
        serde_json::from_str(&data).map_err(|err| OpenAiCompatibleError::InvalidStreamEvent {
            message: format!("invalid JSON chat stream data: {err}"),
        })?;

    for choice in value
        .get("choices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(delta) = choice.get("delta") {
            record_openai_chat_tool_deltas(delta, tool_calls);
            if let Some(reasoning) = chat_reasoning_delta(delta) {
                events.push(ModelStreamEvent::ReasoningSummaryDelta {
                    text: reasoning.to_string(),
                });
            }
        }
        if let Some(delta) = choice
            .get("delta")
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
        {
            events.push(ModelStreamEvent::Delta {
                text: delta.to_string(),
            });
        }
        if let Some(finish_reason) = choice.get("finish_reason").and_then(Value::as_str) {
            flush_openai_chat_tool_calls(tool_calls, events)?;
            let output_token_count = value
                .get("usage")
                .and_then(|usage| usage.get("completion_tokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .min(u64::from(u32::MAX)) as u32;
            events.push(ModelStreamEvent::Completed {
                finish_reason: finish_reason.to_string(),
                output_token_count,
            });
        }
    }

    Ok(())
}

fn chat_reasoning_delta(delta: &Value) -> Option<&str> {
    delta
        .get("reasoning_content")
        .or_else(|| delta.get("reasoning"))
        .or_else(|| delta.get("thinking"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}

#[derive(Clone, Debug, Default)]
struct OpenAiChatToolCallState {
    call_id: Option<String>,
    tool_name: Option<String>,
    arguments: String,
}

#[derive(Clone, Debug, Default)]
struct OpenAiResponseToolCallState {
    call_id: Option<String>,
    tool_name: Option<String>,
    arguments: String,
}

fn record_openai_chat_tool_deltas(
    delta: &Value,
    tool_calls: &mut BTreeMap<u64, OpenAiChatToolCallState>,
) {
    for tool_call in delta
        .get("tool_calls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let index = tool_call.get("index").and_then(Value::as_u64).unwrap_or(0);
        let state = tool_calls.entry(index).or_default();
        if let Some(call_id) = tool_call.get("id").and_then(Value::as_str) {
            state.call_id = Some(call_id.to_string());
        }
        if let Some(function) = tool_call.get("function") {
            if let Some(name) = function.get("name").and_then(Value::as_str) {
                state.tool_name = Some(internal_tool_name_from_openai(name));
            }
            if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                state.arguments.push_str(arguments);
            }
        }
    }
}

fn flush_openai_chat_tool_calls(
    tool_calls: &mut BTreeMap<u64, OpenAiChatToolCallState>,
    events: &mut ModelStreamResult,
) -> Result<(), OpenAiCompatibleError> {
    let pending = std::mem::take(tool_calls);
    for (index, state) in pending {
        let Some(tool_name) = state.tool_name else {
            continue;
        };
        events.push(ModelStreamEvent::ToolCall {
            call_id: state
                .call_id
                .unwrap_or_else(|| format!("openai-chat-call-{index}")),
            tool_name,
            input_json: parse_tool_arguments(&state.arguments)?,
        });
    }
    Ok(())
}

fn record_openai_response_tool_item(
    item: &Value,
    tool_calls: &mut BTreeMap<String, OpenAiResponseToolCallState>,
) {
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return;
    }
    let key = item
        .get("id")
        .or_else(|| item.get("call_id"))
        .and_then(Value::as_str)
        .unwrap_or("response-call-0")
        .to_string();
    let state = tool_calls.entry(key).or_default();
    if let Some(call_id) = item.get("call_id").and_then(Value::as_str) {
        state.call_id = Some(call_id.to_string());
    }
    if let Some(name) = item.get("name").and_then(Value::as_str) {
        state.tool_name = Some(internal_tool_name_from_openai(name));
    }
    if let Some(arguments) = item.get("arguments").and_then(Value::as_str) {
        state.arguments.push_str(arguments);
    }
}

fn response_tool_call_key(value: &Value) -> String {
    value
        .get("item_id")
        .or_else(|| value.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            value
                .get("output_index")
                .and_then(Value::as_u64)
                .map(|index| format!("output-{index}"))
                .unwrap_or_else(|| "response-call-0".to_string())
        })
}

fn openai_response_tool_event_from_item(
    item: &Value,
) -> Result<Option<ModelStreamEvent>, OpenAiCompatibleError> {
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return Ok(None);
    }
    let Some(tool_name) = item.get("name").and_then(Value::as_str) else {
        return Ok(None);
    };
    let call_id = item
        .get("call_id")
        .or_else(|| item.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("response-call-0")
        .to_string();
    let arguments = item
        .get("arguments")
        .and_then(Value::as_str)
        .unwrap_or("{}");
    Ok(Some(ModelStreamEvent::ToolCall {
        call_id,
        tool_name: internal_tool_name_from_openai(tool_name),
        input_json: parse_tool_arguments(arguments)?,
    }))
}

fn flush_openai_response_tool_calls(
    tool_calls: &mut BTreeMap<String, OpenAiResponseToolCallState>,
    events: &mut ModelStreamResult,
) -> Result<(), OpenAiCompatibleError> {
    let pending = std::mem::take(tool_calls);
    for (key, state) in pending {
        let Some(tool_name) = state.tool_name else {
            continue;
        };
        events.push(ModelStreamEvent::ToolCall {
            call_id: state.call_id.unwrap_or(key),
            tool_name,
            input_json: parse_tool_arguments(&state.arguments)?,
        });
    }
    Ok(())
}

fn parse_tool_arguments(arguments: &str) -> Result<Value, OpenAiCompatibleError> {
    let arguments = if arguments.trim().is_empty() {
        "{}"
    } else {
        arguments
    };
    let value: Value = serde_json::from_str(arguments).map_err(|err| {
        OpenAiCompatibleError::InvalidStreamEvent {
            message: format!("invalid tool arguments JSON: {err}"),
        }
    })?;
    if !value.is_object() {
        return Err(OpenAiCompatibleError::InvalidStreamEvent {
            message: "tool arguments JSON must be an object".to_string(),
        });
    }
    Ok(value)
}

fn openai_compatible_tool_name(name: &str) -> String {
    name.replace('.', "__")
}

fn internal_tool_name_from_openai(name: &str) -> String {
    name.replace("__", ".")
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum OpenAiCompatibleError {
    #[error("OpenAI-compatible provider {provider_id} requires a secret reference")]
    MissingSecretRef { provider_id: String },
    #[error("model provider {provider_id} is not OpenAI-compatible")]
    InvalidProviderKind { provider_id: String },
    #[error("invalid OpenAI-compatible provider config: {message}")]
    InvalidConfig { message: String },
    #[error("OpenAI-compatible secret resolution failed: {message}")]
    SecretResolution { message: String },
    #[error("OpenAI-compatible transport failed: {message}")]
    Transport { message: String },
    #[error("OpenAI-compatible provider returned status {status}: {body_summary}")]
    ProviderResponse { status: u16, body_summary: String },
    #[error("invalid OpenAI-compatible stream event: {message}")]
    InvalidStreamEvent { message: String },
}

impl OpenAiCompatibleError {
    #[must_use]
    pub fn provider_response(status: u16, body: &str) -> Self {
        Self::ProviderResponse {
            status,
            body_summary: redact_secret_like(body),
        }
    }

    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingSecretRef { .. } => "model.openai_compatible.missing_secret_ref",
            Self::InvalidProviderKind { .. } => "model.openai_compatible.invalid_provider_kind",
            Self::InvalidConfig { .. } => "model.openai_compatible.invalid_config",
            Self::SecretResolution { .. } => "model.openai_compatible.secret_resolution",
            Self::Transport { .. } => "model.openai_compatible.transport",
            Self::ProviderResponse { .. } => "model.openai_compatible.provider_response",
            Self::InvalidStreamEvent { .. } => "model.openai_compatible.invalid_stream_event",
        }
    }

    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::Transport { .. } | Self::InvalidStreamEvent { .. } => true,
            Self::ProviderResponse { status, .. } => *status == 429 || *status >= 500,
            Self::MissingSecretRef { .. }
            | Self::InvalidProviderKind { .. }
            | Self::InvalidConfig { .. }
            | Self::SecretResolution { .. } => false,
        }
    }
}

impl From<OpenAiCompatibleError> for ModelError {
    fn from(error: OpenAiCompatibleError) -> Self {
        ModelError::Provider {
            code: error.code().to_string(),
            message: error.to_string(),
            retryable: error.retryable(),
        }
    }
}

fn openai_compatible_responses_url(config: &ModelProviderConfig) -> String {
    let base_url = config
        .endpoint
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_OPENAI_COMPATIBLE_BASE_URL)
        .trim_end_matches('/');
    format!("{base_url}/responses")
}

fn openai_compatible_chat_completions_url(config: &ModelProviderConfig) -> String {
    let base_url = config
        .endpoint
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_OPENAI_COMPATIBLE_BASE_URL)
        .trim_end_matches('/');
    if base_url.ends_with("/v1") {
        format!("{base_url}/chat/completions")
    } else {
        format!("{base_url}/v1/chat/completions")
    }
}

fn should_try_chat_completions_fallback(error: &OpenAiCompatibleError) -> bool {
    match error {
        OpenAiCompatibleError::Transport { .. }
        | OpenAiCompatibleError::InvalidStreamEvent { .. } => true,
        OpenAiCompatibleError::ProviderResponse { status, .. } => {
            matches!(*status, 404 | 405 | 503) || *status >= 500
        }
        OpenAiCompatibleError::MissingSecretRef { .. }
        | OpenAiCompatibleError::InvalidProviderKind { .. }
        | OpenAiCompatibleError::InvalidConfig { .. }
        | OpenAiCompatibleError::SecretResolution { .. } => false,
    }
}

fn parse_sse_frame(frame: &str) -> Option<(Option<String>, String)> {
    let mut event_name = None;
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        if let Some(raw_event) = line.strip_prefix("event:") {
            event_name = Some(raw_event.trim().to_string());
        } else if let Some(raw_data) = line.strip_prefix("data:") {
            data_lines.push(raw_data.trim().to_string());
        }
    }

    if data_lines.is_empty() {
        None
    } else {
        Some((event_name, data_lines.join("\n")))
    }
}

fn next_sse_frame_boundary(input: &str) -> Option<(usize, usize)> {
    let lf = input.find("\n\n").map(|index| (index, index + 2));
    let crlf = input.find("\r\n\r\n").map(|index| (index, index + 4));

    match (lf, crlf) {
        (Some(lf), Some(crlf)) => Some(if lf.0 < crlf.0 { lf } else { crlf }),
        (Some(boundary), None) | (None, Some(boundary)) => Some(boundary),
        (None, None) => None,
    }
}

fn is_incomplete_json_stream_event(error: &OpenAiCompatibleError) -> bool {
    matches!(
        error,
        OpenAiCompatibleError::InvalidStreamEvent { message }
            if message.contains("EOF while parsing")
    )
}

fn redact_secret_like(input: &str) -> String {
    let mut redacted = input.to_string();
    for token in input.split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))) {
        let lower = token.to_ascii_lowercase();
        if token.starts_with("sk-")
            || lower.contains("api_key")
            || lower.contains("token")
            || lower.contains("password")
        {
            redacted = redacted.replace(token, "[redacted]");
        }
    }
    if redacted.len() > 240 {
        redacted.truncate(240);
        redacted.push_str("...");
    }
    redacted
}

fn sanitize_url_in_message(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            if part.starts_with("http://") || part.starts_with("https://") {
                sanitize_url(part)
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_url(input: &str) -> String {
    let Some((scheme, rest)) = input.split_once("://") else {
        return input.to_string();
    };
    let rest = rest.split_once('@').map(|(_, tail)| tail).unwrap_or(rest);
    let (host_path, query_suffix) = match rest.split_once('?') {
        Some((host_path, _query)) => (host_path, "?<redacted>"),
        None => (rest, ""),
    };
    format!("{scheme}://{host_path}{query_suffix}")
}
