use std::time::Instant;

use ruse_ui_bridge::{
    UiDiagnosticLevel, UiEventPayload, UiJsonFrame, UiToolPolicySummary, UiToolSandboxSummary,
};

const DEFAULT_TRANSCRIPT_LIMIT: usize = 500;
const MODEL_PENDING_PLACEHOLDER: &str = "thinking";
const REASONING_PENDING_PLACEHOLDER: &str = "thinking";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TranscriptItemKind {
    User,
    Reasoning,
    Model,
    ToolCall,
    ToolResult,
    Approval,
    Diagnostic,
    Progress,
}

impl TranscriptItemKind {
    /// Map to the shared UI taxonomy (plain REPL + TUI).
    #[must_use]
    pub const fn ui_kind(self) -> ruse_ui_bridge::UiTranscriptKind {
        match self {
            Self::User => ruse_ui_bridge::UiTranscriptKind::User,
            Self::Reasoning => ruse_ui_bridge::UiTranscriptKind::Reasoning,
            Self::Model => ruse_ui_bridge::UiTranscriptKind::Assistant,
            Self::ToolCall => ruse_ui_bridge::UiTranscriptKind::ToolCall,
            Self::ToolResult => ruse_ui_bridge::UiTranscriptKind::ToolResult,
            Self::Approval => ruse_ui_bridge::UiTranscriptKind::Approval,
            Self::Diagnostic => ruse_ui_bridge::UiTranscriptKind::Diagnostic,
            Self::Progress => ruse_ui_bridge::UiTranscriptKind::Progress,
        }
    }
}

/// The kind of work a turn is doing while the user waits for output. Drives the
/// label shown next to the animated waiting indicator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WaitingActivity {
    /// A turn has started but no model or reasoning output has arrived yet.
    Waiting,
    /// The model is streaming reasoning/thinking output.
    Thinking,
    /// The model is streaming its final answer.
    Responding,
    /// A tool call is running.
    Working,
}

impl WaitingActivity {
    /// Human-facing verb shown next to the spinner, e.g. `Thinking`.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            WaitingActivity::Waiting => "Waiting",
            WaitingActivity::Thinking => "Thinking",
            WaitingActivity::Responding => "Responding",
            WaitingActivity::Working => "Working",
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ReasoningVisibility {
    /// Stay collapsed until the user expands it (Ctrl+P). Streaming no longer
    /// auto-opens the body — only the thinking header is shown by default.
    #[default]
    Auto,
    /// Keep this reasoning item expanded until explicitly changed.
    Expanded,
    /// Keep this reasoning item collapsed until explicitly changed.
    Collapsed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptItem {
    pub kind: TranscriptItemKind,
    pub title: String,
    pub body: String,
    pub streaming: bool,
    pub metadata_id: Option<String>,
    pub reasoning_visibility: ReasoningVisibility,
    /// When true, long tool output is fully expanded (Ctrl+O). Default false so
    /// shell/tool results stay compact (2 visible lines) until the user asks.
    pub tool_output_expanded: bool,
    /// Wall-clock origin for live thinking duration. Set when a reasoning block
    /// starts streaming; cleared once the duration is frozen.
    pub thinking_started_at: Option<Instant>,
    /// Thinking duration in whole seconds. Live while streaming (refreshed by the
    /// event loop), frozen when the reasoning block completes.
    pub thinking_seconds: u64,
    /// Estimated thinking tokens for the reasoning body (`chars/4`). Updated as
    /// the summary streams so the header can show a live count.
    pub thinking_tokens: u32,
}

impl TranscriptItem {
    #[must_use]
    pub fn new(
        kind: TranscriptItemKind,
        title: impl Into<String>,
        body: impl Into<String>,
        streaming: bool,
    ) -> Self {
        let body = body.into();
        let is_reasoning = kind == TranscriptItemKind::Reasoning;
        let thinking_tokens = if is_reasoning {
            estimate_thinking_tokens(&body)
        } else {
            0
        };
        Self {
            kind,
            title: title.into(),
            body,
            streaming,
            metadata_id: None,
            reasoning_visibility: ReasoningVisibility::Auto,
            tool_output_expanded: false,
            // Synthetic/test items start with no clock; live streams get an origin
            // when the first reasoning delta arrives (or on the next spinner tick).
            thinking_started_at: if is_reasoning && streaming {
                Some(Instant::now())
            } else {
                None
            },
            thinking_seconds: 0,
            thinking_tokens,
        }
    }

    #[must_use]
    pub fn streaming_model(body: impl Into<String>) -> Self {
        Self::new(TranscriptItemKind::Model, "assistant", body, true)
    }

    #[must_use]
    pub fn completed_model(body: impl Into<String>) -> Self {
        Self::new(TranscriptItemKind::Model, "assistant", body, false)
    }

    #[must_use]
    pub fn tool_call(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        status: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        let call_id = call_id.into();
        let tool_name = tool_name.into();
        let status = status.into();
        let detail = detail.into();
        let body = tool_call_body(&tool_name, &detail);
        Self {
            kind: TranscriptItemKind::ToolCall,
            title: "tool".to_string(),
            body,
            streaming: status == "running",
            metadata_id: Some(call_id),
            reasoning_visibility: ReasoningVisibility::Auto,
            tool_output_expanded: false,
            thinking_started_at: None,
            thinking_seconds: 0,
            thinking_tokens: 0,
        }
    }

    /// Refresh live thinking duration/tokens for a streaming reasoning item.
    pub fn refresh_thinking_stats(&mut self, now: Instant) {
        if self.kind != TranscriptItemKind::Reasoning {
            return;
        }
        self.thinking_tokens = estimate_thinking_tokens(&self.body);
        if !self.streaming {
            return;
        }
        let started = *self.thinking_started_at.get_or_insert(now);
        self.thinking_seconds = now.duration_since(started).as_secs();
    }

    /// Freeze thinking duration when a reasoning stream ends.
    pub fn freeze_thinking_stats(&mut self, now: Instant) {
        if self.kind != TranscriptItemKind::Reasoning {
            return;
        }
        self.thinking_tokens = estimate_thinking_tokens(&self.body);
        if let Some(started) = self.thinking_started_at.take() {
            self.thinking_seconds = now.duration_since(started).as_secs();
        }
    }

    #[must_use]
    pub fn is_reasoning_collapsed(&self) -> bool {
        if self.kind != TranscriptItemKind::Reasoning {
            return false;
        }
        match self.reasoning_visibility {
            // Default/auto: always collapsed. Only an explicit Expanded choice
            // (Ctrl+P / /thinking show) reveals the body.
            ReasoningVisibility::Auto | ReasoningVisibility::Collapsed => true,
            ReasoningVisibility::Expanded => false,
        }
    }

    pub fn set_reasoning_collapsed(&mut self, collapsed: bool) {
        if self.kind == TranscriptItemKind::Reasoning {
            self.reasoning_visibility = if collapsed {
                ReasoningVisibility::Collapsed
            } else {
                ReasoningVisibility::Expanded
            };
        }
    }

    pub fn toggle_reasoning(&mut self) {
        let collapsed = !self.is_reasoning_collapsed();
        self.set_reasoning_collapsed(collapsed);
    }

    /// Toggle expansion of the latest tool call/result that has multi-line
    /// output. Returns true when a tool item was found and toggled.
    pub fn toggle_tool_output(&mut self) -> bool {
        if !matches!(
            self.kind,
            TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
        ) {
            return false;
        }
        self.tool_output_expanded = !self.tool_output_expanded;
        true
    }

    #[must_use]
    pub fn render_line(&self) -> String {
        let state = if self.streaming { " streaming" } else { "" };
        format!("{}{}: {}", self.title, state, self.body)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptState {
    pub items: Vec<TranscriptItem>,
    pub scroll_offset: u16,
    limit: usize,
    /// An approval request paused the current reasoning stream. The next real
    /// reasoning delta starts on a new rendered line, while the same Thinking /
    /// Thought item remains open across the tool interaction.
    reasoning_break_pending: bool,
}

impl TranscriptState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            items: Vec::new(),
            scroll_offset: 0,
            limit: limit.max(1),
            reasoning_break_pending: false,
        }
    }

    pub fn push(&mut self, item: TranscriptItem) {
        self.items.push(item);
        self.scroll_offset = 0;
        self.enforce_limit();
    }

    pub fn scroll_up(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    pub fn scroll_down(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = u16::MAX;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn toggle_latest_reasoning(&mut self) -> bool {
        let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Reasoning)
        else {
            return false;
        };
        item.toggle_reasoning();
        true
    }

    /// Expand/collapse the latest tool call/result output. Used by Ctrl+O so
    /// long shell results stay compact by default (2 lines) until requested.
    pub fn toggle_latest_tool_output(&mut self) -> bool {
        let Some(item) = self.items.iter_mut().rev().find(|item| {
            matches!(
                item.kind,
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
            )
        }) else {
            return false;
        };
        item.toggle_tool_output()
    }

    pub fn set_all_reasoning_collapsed(&mut self, collapsed: bool) -> usize {
        let mut changed = 0;
        for item in &mut self.items {
            if item.kind == TranscriptItemKind::Reasoning {
                item.set_reasoning_collapsed(collapsed);
                changed += 1;
            }
        }
        changed
    }

    #[must_use]
    pub fn latest_completed_model_body(&self) -> Option<&str> {
        self.items
            .iter()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Model && !item.streaming)
            .map(|item| item.body.as_str())
    }

    /// Derive the current waiting activity from the streaming state of the
    /// transcript. Returns `None` when nothing is in flight, in which case no
    /// waiting indicator should be shown.
    #[must_use]
    pub fn waiting_activity(&self) -> Option<WaitingActivity> {
        self.items
            .iter()
            .rev()
            .find(|item| item.streaming)
            .map(|item| match item.kind {
                TranscriptItemKind::ToolCall => WaitingActivity::Working,
                TranscriptItemKind::Model => {
                    if item.body == MODEL_PENDING_PLACEHOLDER {
                        WaitingActivity::Waiting
                    } else {
                        WaitingActivity::Responding
                    }
                }
                _ => WaitingActivity::Thinking,
            })
    }

    pub fn reduce_frame(&mut self, frame: UiJsonFrame) {
        if !matches!(
            &frame,
            UiJsonFrame::Event(_)
                | UiJsonFrame::ConfigDiagnostic { .. }
                | UiJsonFrame::Error { .. }
        ) {
            return;
        }

        match frame {
            UiJsonFrame::Event(event) => self.reduce_event_payload(event.payload),
            UiJsonFrame::ConfigDiagnostic {
                severity,
                code,
                message,
                ..
            } => {
                if severity == "error" {
                    self.remove_pending_model_placeholder();
                    self.remove_pending_reasoning_placeholder();
                }
                self.push(TranscriptItem::new(
                    TranscriptItemKind::Diagnostic,
                    code,
                    format!("{severity}: {message}"),
                    false,
                ));
            }
            UiJsonFrame::Error {
                code,
                message,
                recoverable,
            } => {
                self.remove_pending_model_placeholder();
                self.remove_pending_reasoning_placeholder();
                self.push(TranscriptItem::new(
                    TranscriptItemKind::Diagnostic,
                    code,
                    format!(
                        "{} ({})",
                        message,
                        if recoverable {
                            "recoverable"
                        } else {
                            "terminal"
                        }
                    ),
                    false,
                ));
            }
            // Every other frame is surfaced through the status footer, the
            // Status side panel, or the live activity line — not the transcript.
            // (The `reduce_frame` guard above already lets only Event/
            // ConfigDiagnostic/Error through, so this is defensive.)
            _ => {}
        }
    }

    fn reduce_event_payload(&mut self, payload: UiEventPayload) {
        match payload {
            UiEventPayload::UserMessage { message } => {
                self.reasoning_break_pending = false;
                self.push(TranscriptItem::new(
                    TranscriptItemKind::User,
                    "user",
                    message,
                    false,
                ));
            }
            UiEventPayload::ReasoningSummaryDelta { delta } => self.append_reasoning_delta(&delta),
            UiEventPayload::ModelDelta { delta } => self.append_model_delta(&delta),
            UiEventPayload::FinalResponse { message } => self.close_model_stream(message),
            UiEventPayload::ToolCall {
                call_id,
                tool_name,
                preview,
            } => {
                self.close_streams_before_tool();
                let detail = preview.unwrap_or_default();
                self.upsert_tool_call(call_id, tool_name, "proposed", &detail);
            }
            UiEventPayload::ToolStarted {
                call_id,
                tool_name,
                policy,
                sandbox,
            } => {
                self.close_streams_before_tool();
                self.upsert_tool_call(
                    call_id,
                    tool_name,
                    "running",
                    &tool_start_detail(policy.as_ref(), sandbox.as_ref()),
                );
            }
            UiEventPayload::ToolObservation {
                call_id,
                observation,
            } => {
                // Observation is emitted after ToolResult for model context. Never
                // reopen a finished tool as Running; only attach cleaned detail if
                // the running item still needs user-facing output.
                self.update_tool_observation(&call_id, &observation);
            }
            UiEventPayload::ToolResult {
                call_id,
                ok,
                summary,
                elapsed_ms,
            } => {
                self.close_streams_before_tool();
                let detail = tool_result_detail(&summary, elapsed_ms);
                if !self.finish_tool_with_detail(&call_id, &detail) {
                    let status = if ok { "ok" } else { "failed" };
                    let body = if detail.is_empty() {
                        format!("{status} {call_id}")
                    } else {
                        format!("{status} {call_id}\n{detail}")
                    };
                    let mut item =
                        TranscriptItem::new(TranscriptItemKind::ToolResult, "tool", body, false);
                    item.metadata_id = Some(call_id);
                    if tool_result_should_auto_expand("shell", &detail) || !detail.is_empty() {
                        // Prefer showing the finished result rather than hiding it.
                        item.tool_output_expanded = true;
                    }
                    self.push(item);
                }
            }
            UiEventPayload::ApprovalRequested(_) => {
                // Pending approvals live in the modal + footer indicator only.
                // Preserve the single reasoning item, but make the first summary
                // delta after this user-visible pause start on a fresh line.
                self.reasoning_break_pending |= self.has_continuable_reasoning();
            }
            UiEventPayload::ApprovalResolved { .. } => {
                // Do not echo allow/reject into the transcript chat stream.
                // Decision UX is the approval modal; repeating it is noise.
            }
            UiEventPayload::Diagnostic {
                level,
                code,
                message,
                recoverable,
            } => {
                if level == UiDiagnosticLevel::Error {
                    self.remove_pending_model_placeholder();
                    self.remove_pending_reasoning_placeholder();
                }
                self.push(TranscriptItem::new(
                    TranscriptItemKind::Diagnostic,
                    code,
                    format!(
                        "{}: {message}{}",
                        diagnostic_level_name(level),
                        if recoverable { " (recoverable)" } else { "" }
                    ),
                    false,
                ));
            }
            UiEventPayload::Progress { .. }
            | UiEventPayload::ContextStatus { .. }
            | UiEventPayload::TurnState { .. }
            | UiEventPayload::Permission { .. }
            | UiEventPayload::Checkpoint { .. }
            | UiEventPayload::PromptSummary(_)
            | UiEventPayload::TaskNotification(_)
            | UiEventPayload::ExternalTrace { .. }
            | UiEventPayload::Workspace { .. }
            | UiEventPayload::Session { .. } => {}
        }
    }

    fn append_model_delta(&mut self, delta: &str) {
        self.reasoning_break_pending = false;
        self.close_or_remove_reasoning_placeholder();
        // Drop pure tool_call wire-format fragments so they never paint as chat.
        // (Runtime also recovers these into real ToolCall events.)
        if is_tool_call_wire_fragment(delta) {
            return;
        }
        if let Some(item) = self
            .items
            .last_mut()
            .filter(|item| item.kind == TranscriptItemKind::Model && item.streaming)
        {
            if item.body == MODEL_PENDING_PLACEHOLDER {
                item.body.clear();
            }
            item.body.push_str(delta);
            // If the accumulated body is only a tool_call dump, blank it so the
            // row can be dropped at close time.
            if is_tool_call_only_body(&item.body) {
                item.body.clear();
            }
            return;
        }

        // Empty deltas must not open a ghost model item — that would paint as a
        // blank row and inflate inter-item spacing once a real message arrives.
        if delta.is_empty() || is_tool_call_only_body(delta) {
            return;
        }

        self.close_model_stream_without_final();
        self.push(TranscriptItem::streaming_model(delta));
    }

    fn append_reasoning_delta(&mut self, delta: &str) {
        // Empty / whitespace deltas are noise: they must not clear the turn-start
        // placeholder into a ghost empty body that later freezes as
        // "Thought [0s · 0 tokens]".
        if delta.trim().is_empty() {
            return;
        }

        self.remove_pending_model_placeholder();
        // Drop a closed empty model that may have been left behind after a tool
        // boundary; it would only contribute blank spacing in the transcript.
        self.remove_empty_trailing_model();
        let break_before_delta = std::mem::take(&mut self.reasoning_break_pending);

        // Continue any open streaming reasoning block — even if tools were
        // inserted after it — so interleaved tool calls don't freeze Thinking
        // into Thought and then spawn a second Thinking row (visual flash).
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Reasoning && item.streaming)
        {
            append_reasoning_segment(item, delta, break_before_delta);
            return;
        }

        // Reopen the most recent reasoning when only tools sit after it. Same
        // row flips Thought → Thinking instead of hiding one and creating another.
        if let Some(index) = self.reopenable_reasoning_index() {
            let item = &mut self.items[index];
            item.streaming = true;
            // Keep the original wall-clock origin so duration stays continuous
            // across tool-interrupted reasoning segments.
            if item.thinking_started_at.is_none() {
                item.thinking_started_at = Some(Instant::now());
            }
            append_reasoning_segment(item, delta, break_before_delta);
            return;
        }

        self.close_reasoning_stream();
        self.close_model_stream_without_final();
        self.remove_empty_trailing_model();
        self.push(TranscriptItem::new(
            TranscriptItemKind::Reasoning,
            "thinking",
            delta,
            true,
        ));
    }

    /// Whether the latest turn still has a reasoning block that may continue
    /// after an approval-gated tool interaction. Tool and diagnostic rows do not
    /// end the reasoning segment; user/model rows do.
    fn has_continuable_reasoning(&self) -> bool {
        for item in self.items.iter().rev() {
            match item.kind {
                TranscriptItemKind::ToolCall
                | TranscriptItemKind::ToolResult
                | TranscriptItemKind::Approval
                | TranscriptItemKind::Diagnostic
                | TranscriptItemKind::Progress => continue,
                TranscriptItemKind::Reasoning => {
                    return !is_empty_reasoning_body(&item.body);
                }
                TranscriptItemKind::User | TranscriptItemKind::Model => return false,
            }
        }
        false
    }

    /// Index of a finished reasoning block that can be reopened because only
    /// tool rows (if any) follow it — no model/user text in between.
    fn reopenable_reasoning_index(&self) -> Option<usize> {
        for (index, item) in self.items.iter().enumerate().rev() {
            match item.kind {
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => continue,
                TranscriptItemKind::Reasoning if !item.streaming => return Some(index),
                _ => return None,
            }
        }
        None
    }

    fn remove_pending_model_placeholder(&mut self) {
        if self.items.last().is_some_and(|item| {
            item.kind == TranscriptItemKind::Model
                && item.streaming
                && item.body == MODEL_PENDING_PLACEHOLDER
        }) {
            self.items.pop();
        }
    }

    /// Remove a trailing model item whose body is empty / whitespace-only. These
    /// are ghost streams that only inflate blank spacing between real items.
    fn remove_empty_trailing_model(&mut self) {
        if self.items.last().is_some_and(|item| {
            item.kind == TranscriptItemKind::Model && item.body.trim().is_empty()
        }) {
            self.items.pop();
        }
    }

    fn upsert_tool_call(&mut self, call_id: String, tool_name: String, status: &str, detail: &str) {
        // Keep the body free of call-id / policy / sandbox noise. The head is
        // just the friendly tool name + target; result lines come later.
        //
        // Phase machine for tools in the chat stream:
        //   proposed  → Waiting (ToolCall, !streaming)
        //   running   → Running (ToolCall, streaming)
        //   result    → Done    (ToolResult, !streaming) via finish_tool_with_detail
        let body = tool_call_body(&tool_name, detail);
        let running = status == "running";
        if !self.update_tool_by_id(&call_id, body, running) {
            self.push(TranscriptItem::tool_call(
                call_id, tool_name, status, detail,
            ));
        }
    }

    fn update_tool_by_id(&mut self, call_id: &str, body: String, running: bool) -> bool {
        let Some(item) = self.items.iter_mut().rev().find(|item| {
            matches!(
                item.kind,
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
            ) && item.metadata_id.as_deref() == Some(call_id)
        }) else {
            return false;
        };

        // Never demote a finished tool back to waiting/running via upsert.
        if item.kind == TranscriptItemKind::ToolResult && !item.streaming {
            return true;
        }

        // Waiting and Running both stay ToolCall. Only finish_tool_with_detail
        // promotes to ToolResult — so proposed tools no longer render as Ran.
        item.kind = TranscriptItemKind::ToolCall;
        item.title = "tool".to_string();
        // Preserve an existing head (friendly name + target) when the new body
        // is just a status/policy blob with no useful target, or a poorer head
        // that drops the path/command already shown.
        if should_replace_tool_body(&item.body, &body) {
            item.body = body;
        }
        item.streaming = running;
        true
    }

    /// Attach cleaned observation text without reopening a finished tool.
    fn update_tool_observation(&mut self, call_id: &str, observation: &str) -> bool {
        let Some(item) = self.items.iter_mut().rev().find(|item| {
            matches!(
                item.kind,
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
            ) && item.metadata_id.as_deref() == Some(call_id)
        }) else {
            return false;
        };

        let cleaned = clean_tool_result_detail(observation);
        if cleaned.is_empty() {
            return true;
        }
        let prefix = tool_body_prefix(&item.body);
        let finished = item.kind == TranscriptItemKind::ToolResult && !item.streaming;
        if finished {
            // Completed already — only replace body when the current detail is
            // empty or still looks like raw tool JSON noise.
            let current_detail = item.body.split_once('\n').map(|(_, d)| d).unwrap_or("");
            if current_detail.trim().is_empty() || looks_like_raw_tool_payload(current_detail) {
                item.body = format!("{prefix}\n{cleaned}");
                if tool_result_should_auto_expand(&prefix, &cleaned) {
                    item.tool_output_expanded = true;
                }
            }
            return true;
        }

        // Still live: show cleaned output but stay Running until ToolResult.
        item.body = format!("{prefix}\n{cleaned}");
        item.kind = TranscriptItemKind::ToolCall;
        item.streaming = true;
        true
    }

    fn finish_tool_with_detail(&mut self, call_id: &str, detail: &str) -> bool {
        let Some(item) = self.items.iter_mut().rev().find(|item| {
            matches!(
                item.kind,
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
            ) && item.metadata_id.as_deref() == Some(call_id)
        }) else {
            return false;
        };

        let prefix = tool_body_prefix(&item.body);
        let cleaned = clean_tool_result_detail(detail);
        item.kind = TranscriptItemKind::ToolResult;
        item.title = "tool".to_string();
        item.streaming = false;
        item.body = if cleaned.is_empty() {
            prefix.clone()
        } else {
            format!("{prefix}\n{cleaned}")
        };
        // File edits (diff) and shell/command output expand so the user sees
        // the result without an extra click.
        if tool_result_should_auto_expand(&prefix, &cleaned)
            || (!cleaned.is_empty() && prefix.starts_with("shell"))
        {
            item.tool_output_expanded = true;
        }
        true
    }

    fn close_model_stream(&mut self, message: String) {
        self.reasoning_break_pending = false;
        self.close_or_remove_reasoning_placeholder();
        let cleaned = strip_tool_call_wire_text(&message);
        // Tool-call-only finals become empty — drop the model row entirely so the
        // next Ran/Edited tool line is the first useful content.
        if cleaned.trim().is_empty() {
            if let Some(last) = self.items.last()
                && last.kind == TranscriptItemKind::Model
                && last.streaming
            {
                self.items.pop();
                return;
            }
            // No streaming model item; nothing useful to add.
            if is_tool_call_only_body(&message) || message.trim().is_empty() {
                return;
            }
        }
        let body = if cleaned.trim().is_empty() {
            "done".to_string()
        } else {
            cleaned
        };
        if let Some(item) = self
            .items
            .last_mut()
            .filter(|item| item.kind == TranscriptItemKind::Model && item.streaming)
        {
            item.body = body;
            item.streaming = false;
            return;
        }

        self.close_model_stream_without_final();
        self.push(TranscriptItem::completed_model(body));
    }

    fn close_streams_before_tool(&mut self) {
        // Drop only an unused turn-start placeholder. Keep real reasoning open
        // across tool execution so the Thinking row does not freeze to Thought
        // and then reappear — that hide/show is the flash users notice mid-turn.
        self.remove_pending_reasoning_placeholder();
        self.remove_pending_model_placeholder();
        self.close_model_stream_without_final();
        self.remove_empty_trailing_model();
    }

    fn close_model_stream_without_final(&mut self) {
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Model && item.streaming)
        {
            item.streaming = false;
        }
        // A closed empty model is a spacing ghost — drop it so the next real item
        // doesn't sit two separators away from the previous content.
        self.remove_empty_trailing_model();
    }

    fn close_or_remove_reasoning_placeholder(&mut self) {
        if self.remove_pending_or_empty_reasoning() {
            return;
        }
        self.close_reasoning_stream();
    }

    fn close_reasoning_stream(&mut self) {
        // Empty / placeholder reasoning should never settle as "Thought [0s · 0
        // tokens]" — that row has no useful content and is pure noise.
        if self.remove_pending_or_empty_reasoning() {
            return;
        }
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Reasoning && item.streaming)
        {
            item.streaming = false;
            item.freeze_thinking_stats(Instant::now());
        }
    }

    /// Tick live thinking duration/token counters for any streaming reasoning
    /// blocks. Called from the event-loop spinner so headers stay dynamic
    /// without the render layer calling `Instant::now()`.
    pub fn refresh_thinking_stats(&mut self, now: Instant) -> bool {
        let mut changed = false;
        for item in &mut self.items {
            if item.kind != TranscriptItemKind::Reasoning || !item.streaming {
                continue;
            }
            let before = (item.thinking_seconds, item.thinking_tokens);
            item.refresh_thinking_stats(now);
            if (item.thinking_seconds, item.thinking_tokens) != before {
                changed = true;
            }
        }
        changed
    }

    fn remove_pending_reasoning_placeholder(&mut self) -> bool {
        self.remove_pending_or_empty_reasoning()
    }

    /// Drop a streaming reasoning item that still has no real summary text
    /// (turn-start placeholder, empty body after noise deltas, or whitespace).
    fn remove_pending_or_empty_reasoning(&mut self) -> bool {
        if self.items.last().is_some_and(|item| {
            item.kind == TranscriptItemKind::Reasoning
                && item.streaming
                && is_empty_reasoning_body(&item.body)
        }) {
            self.items.pop();
            return true;
        }
        false
    }

    fn enforce_limit(&mut self) {
        let overflow = self.items.len().saturating_sub(self.limit);
        if overflow > 0 {
            self.items.drain(0..overflow);
        }
    }
}

impl Default for TranscriptState {
    fn default() -> Self {
        Self::with_limit(DEFAULT_TRANSCRIPT_LIMIT)
    }
}

/// Rough thinking-token estimate for the reasoning header (`chars / 4`).
/// Matches the lightweight estimator used elsewhere in the runtime so the
/// live counter is directionally useful without a provider usage event.
#[must_use]
pub fn estimate_thinking_tokens(body: &str) -> u32 {
    if is_empty_reasoning_body(body) {
        return 0;
    }
    let trimmed = body.trim();
    u32::try_from(trimmed.chars().count().div_ceil(4).max(1)).unwrap_or(u32::MAX)
}

fn is_empty_reasoning_body(body: &str) -> bool {
    let trimmed = body.trim();
    trimmed.is_empty() || trimmed == REASONING_PENDING_PLACEHOLDER
}

/// Append one reasoning stream segment. Approval pauses use a Markdown hard
/// break (`"  \n"`) so both Markdown and plain rendering put the resumed thought
/// on a new terminal row without splitting it into a second Thought item.
fn append_reasoning_segment(item: &mut TranscriptItem, delta: &str, break_before: bool) {
    if is_empty_reasoning_body(&item.body) {
        item.body.clear();
    }

    if break_before && !item.body.is_empty() {
        item.body.truncate(item.body.trim_end().len());
        item.body.push_str("  \n");
        item.body.push_str(delta.trim_start());
    } else {
        item.body.push_str(delta);
    }
    item.thinking_tokens = estimate_thinking_tokens(&item.body);
}

/// Build the compact body for a tool call: `friendly_name target` (no call id,
/// no policy/sandbox metadata). Result output is appended later as ` - …`.
fn tool_call_body(tool_name: &str, detail: &str) -> String {
    let friendly = friendly_tool_name(tool_name);
    let target = tool_target_from_detail(tool_name, detail);
    if target.is_empty() {
        friendly
    } else {
        format!("{friendly} {target}")
    }
}

/// Map built-in tool ids to short, user-facing names shown in the transcript.
#[must_use]
pub fn friendly_tool_name(tool_name: &str) -> String {
    match tool_name {
        "workspace.read_file" => "read_file".to_string(),
        "workspace.write_file" => "write_file".to_string(),
        "workspace.edit_file" => "edit_file".to_string(),
        "workspace.search_text" => "search".to_string(),
        "workspace.git_status" => "git_status".to_string(),
        "workspace.git_diff" => "git_diff".to_string(),
        "workspace.shell_command" => "shell".to_string(),
        other if other.starts_with("workspace.") => {
            other.trim_start_matches("workspace.").replace('_', " ")
        }
        other if other.starts_with("mcp.") => {
            // mcp.<server>.<tool> → server.tool
            let rest = other.trim_start_matches("mcp.");
            rest.to_string()
        }
        other => other.to_string(),
    }
}

/// Pull a human-facing target (path / command / query) out of a tool detail
/// string. Policy, sandbox, call-id, and elapsed metadata are intentionally
/// dropped — they belong in the status/trace surfaces, not the chat stream.
fn tool_target_from_detail(tool_name: &str, detail: &str) -> String {
    let detail = detail.trim();
    if detail.is_empty() || detail == "started" {
        return String::new();
    }
    // Policy/sandbox start blobs: "started policy=… sandbox=…" — no target.
    if detail.starts_with("started ") && (detail.contains("policy=") || detail.contains("sandbox="))
    {
        return String::new();
    }

    // Prefer structured previews from the bridge (`read_file path`, `run cmd`).
    if let Some(path) = detail
        .strip_prefix("read file ")
        .or_else(|| detail.strip_prefix("write file "))
        .or_else(|| detail.strip_prefix("edit file "))
    {
        return path.trim().to_string();
    }
    if let Some(command) = detail.strip_prefix("run ") {
        return command.trim().to_string();
    }
    if let Some(rest) = detail.strip_prefix("search ") {
        // "search PATH for QUERY" → keep as-is (already short).
        return rest.trim().to_string();
    }
    if let Some(path) = detail
        .strip_prefix("git status ")
        .or_else(|| detail.strip_prefix("git diff "))
    {
        return path.trim().to_string();
    }

    // JSON observation / result blobs often embed a path.
    if let Some(path) = json_string_field(detail, "path") {
        return path;
    }
    if tool_name.ends_with("shell_command") {
        if let Some(program) = json_string_field(detail, "program") {
            let args = json_string_array_field(detail, "args").unwrap_or_default();
            return if args.is_empty() {
                program
            } else {
                format!("{program} {args}")
            };
        }
    }

    // Fall back: if the detail looks like a bare path/command and not a
    // policy blob, show a short form of it.
    if detail.contains('=') && (detail.contains("policy") || detail.contains("sandbox")) {
        return String::new();
    }
    let first_line = detail.lines().next().unwrap_or(detail).trim();
    if first_line.chars().count() > 80 {
        first_line.chars().take(80).collect()
    } else {
        first_line.to_string()
    }
}

fn clean_tool_result_detail(detail: &str) -> String {
    let detail = detail.trim();
    if detail.is_empty() {
        return String::new();
    }
    // Drop elapsed_ms=… suffix — not useful in the chat stream.
    let without_elapsed = detail
        .split_once(" elapsed_ms=")
        .map_or(detail, |(head, _)| head)
        .trim();

    // User-first: for shell-like payloads, only keep the command output.
    // Try strict JSON first, then loose extraction (bridge may truncate to 160
    // chars and break the JSON), then `stdout=` key/value summaries.
    if let Some(stdout) = extract_user_command_output(without_elapsed) {
        return stdout;
    }

    // Prefer a compact path / command / message from common JSON shapes.
    // File edits keep the unified `diff` body so the transcript can expand it.
    if let Some(path) = json_string_field(without_elapsed, "path") {
        if let Some(diff) = json_string_field(without_elapsed, "diff") {
            let trimmed = diff.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(bytes) = json_number_field(without_elapsed, "bytes") {
            return format!("{path} ({bytes} bytes)");
        }
        return path;
    }
    if let Some(diff) = json_string_field(without_elapsed, "diff") {
        let trimmed = diff.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(message) = json_string_field(without_elapsed, "message") {
        return message;
    }
    if let Some(output) = json_string_field(without_elapsed, "output") {
        return output;
    }

    // Strip a leading "observation: " / "result: " label if present.
    let stripped = without_elapsed
        .strip_prefix("observation: ")
        .or_else(|| without_elapsed.strip_prefix("result: "))
        .unwrap_or(without_elapsed)
        .trim();

    // If it still looks like raw JSON noise (sandbox_profile / status / …),
    // never dump it into the chat stream.
    if looks_like_raw_tool_payload(stripped) {
        if let Some(stdout) = extract_json_string_field_loose(stripped, "stdout") {
            return stdout;
        }
        if let Some(stderr) = extract_json_string_field_loose(stripped, "stderr") {
            return stderr;
        }
        return String::new();
    }

    // If it still looks like raw JSON, try to pretty-collapse to a short line.
    if stripped.starts_with('{') {
        if let Some(diff) = json_string_field(stripped, "diff") {
            let trimmed = diff.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(path) = json_string_field(stripped, "path") {
            return path;
        }
    }
    stripped.to_string()
}

/// Pull only what the user cares about from shell / tool result payloads:
/// stdout, else stderr. Drops sandbox_profile, status, exit codes, etc.
fn extract_user_command_output(raw: &str) -> Option<String> {
    if let Some(stdout) = json_string_field(raw, "stdout") {
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(stderr) = json_string_field(raw, "stderr") {
        let trimmed = stderr.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(stdout) = extract_json_string_field_loose(raw, "stdout") {
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(stderr) = extract_json_string_field_loose(raw, "stderr") {
        let trimmed = stderr.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    // Bridge summary shape: `exit=0 stdout=… stderr=…`
    if let Some(stdout) = extract_kv_field(raw, "stdout") {
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(stderr) = extract_kv_field(raw, "stderr") {
        let trimmed = stderr.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn looks_like_raw_tool_payload(raw: &str) -> bool {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('{') || trimmed.contains("sandbox_profile")) {
        return false;
    }
    trimmed.contains("stdout")
        || trimmed.contains("stderr")
        || trimmed.contains("sandbox_profile")
        || trimmed.contains("\"status\"")
}

fn tool_body_prefix(body: &str) -> String {
    body.split_once('\n').map_or_else(
        || {
            body.split_once(" - ")
                .map_or_else(|| body.to_string(), |(prefix, _)| prefix.to_string())
        },
        |(prefix, _)| prefix.to_string(),
    )
}

/// Best-effort string-field extraction that still works when the JSON was
/// truncated mid-string by a UI summary bound.
fn extract_json_string_field_loose(raw: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = raw.find(&marker)? + marker.len();
    let rest = &raw[start..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => break,
            }
        } else if c == '"' {
            break;
        } else {
            out.push(c);
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Extract `key=value` from space-joined summaries. Value runs until the next
/// ` other_key=` token (`exit=`, `stdout=`, `stderr=`, `resource=`, …) or EOL.
fn extract_kv_field(raw: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=");
    let start = raw.find(&marker)? + marker.len();
    let rest = &raw[start..];
    let end = [
        " exit=",
        " stdout=",
        " stderr=",
        " resource=",
        " applied=",
        " files=",
        " timed_out=",
    ]
    .iter()
    .filter_map(|token| rest.find(token))
    .min()
    .unwrap_or(rest.len());
    let value = rest[..end].trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn json_string_field(raw: &str, key: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    value.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn json_number_field(raw: &str, key: &str) -> Option<u64> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    value.get(key).and_then(|v| v.as_u64())
}

fn json_string_array_field(raw: &str, key: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let arr = value.get(key)?.as_array()?;
    let parts: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn diagnostic_level_name(level: UiDiagnosticLevel) -> &'static str {
    match level {
        UiDiagnosticLevel::Warning => "warning",
        UiDiagnosticLevel::Error => "error",
    }
}

fn tool_start_detail(
    _policy: Option<&UiToolPolicySummary>,
    _sandbox: Option<&UiToolSandboxSummary>,
) -> String {
    // Policy/sandbox metadata is intentionally omitted from the chat stream —
    // it belongs in the status/trace surfaces, not next to the tool name.
    String::new()
}

fn tool_result_detail(summary: &str, _elapsed_ms: Option<u64>) -> String {
    // Elapsed timing is noise in the chat stream; keep the summary only.
    clean_tool_result_detail(summary)
}

/// Prefer the richer of two tool-body heads. Prevents a later ToolStarted with
/// no target from erasing `edit_file src/main.rs` down to bare `edit_file`.
fn should_replace_tool_body(existing: &str, incoming: &str) -> bool {
    let incoming = incoming.trim();
    if incoming.is_empty() {
        return false;
    }
    let existing_head = existing
        .split_once('\n')
        .map_or(existing, |(head, _)| head)
        .trim();
    let incoming_head = incoming
        .split_once('\n')
        .map_or(incoming, |(head, _)| head)
        .trim();
    // Incoming has a target (name + path/command); always take it.
    if incoming_head.contains(' ') && !existing_head.contains(' ') {
        return true;
    }
    // Existing already has a target and incoming is just the bare tool name.
    if existing_head.contains(' ') && !incoming_head.contains(' ') {
        return false;
    }
    // Same richness: prefer longer / more specific, otherwise accept update.
    if existing_head.starts_with(incoming_head) && existing_head.len() > incoming_head.len() {
        return false;
    }
    true
}

/// Edit/write tools that return a multi-line diff should open expanded so the
/// change is the primary chat affordance (not a collapsed path-only line).
fn tool_result_should_auto_expand(prefix: &str, detail: &str) -> bool {
    let tool_label = prefix.split_whitespace().next().unwrap_or("");
    if tool_label != "edit_file" && tool_label != "write_file" && tool_label != "git_diff" {
        return false;
    }
    let lines = detail
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    if lines < 2 {
        return false;
    }
    detail.lines().any(|line| {
        line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("@@")
            || (line.starts_with('+') && !line.starts_with("+++"))
            || (line.starts_with('-') && !line.starts_with("---"))
    })
}

/// Detect a streaming delta that is only tool_call wire-format noise.
fn is_tool_call_wire_fragment(delta: &str) -> bool {
    let t = delta.trim();
    if t.is_empty() {
        return false;
    }
    // Short fragments mid-stream: "tool_call", "id=…", "name=workspace…", etc.
    let lower = t.to_ascii_lowercase();
    if lower == "tool_call" || lower.starts_with("tool_call ") || lower.starts_with("tool_call\n") {
        return true;
    }
    if lower.starts_with("id=") && (lower.contains("name=") || lower.contains("input=")) {
        return true;
    }
    is_tool_call_only_body(t)
}

/// True when the body is entirely one or more `tool_call id=… name=… input=…` dumps.
fn is_tool_call_only_body(body: &str) -> bool {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !trimmed.contains("tool_call") && !trimmed.contains("name=workspace.") {
        return false;
    }
    // Every non-empty line looks like wire format.
    trimmed
        .lines()
        .filter(|l| !l.trim().is_empty())
        .all(|line| {
            let l = line.trim().to_ascii_lowercase();
            l.starts_with("tool_call")
                || l.starts_with("id=")
                || (l.contains("name=") && l.contains("input="))
                || l == "tool_call"
        })
}

/// Remove tool_call wire-format lines; keep any surrounding human prose.
fn strip_tool_call_wire_text(body: &str) -> String {
    if !body.contains("tool_call") && !body.contains("name=workspace.") {
        return body.to_string();
    }
    let normalized = body
        .replace("}tool_call", "}\ntool_call")
        .replace("} tool_call", "}\ntool_call");
    let mut kept = Vec::new();
    for line in normalized.lines() {
        let t = line.trim();
        if t.is_empty() {
            if !kept.is_empty() {
                kept.push(String::new());
            }
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower.starts_with("tool_call")
            || (lower.starts_with("id=") && lower.contains("name="))
            || (lower.contains("name=workspace.") && lower.contains("input="))
        {
            continue;
        }
        kept.push(line.to_string());
    }
    while kept.last().is_some_and(|l| l.trim().is_empty()) {
        kept.pop();
    }
    kept.join("\n")
}
