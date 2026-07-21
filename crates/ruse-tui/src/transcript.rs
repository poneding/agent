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
}

impl TranscriptItem {
    #[must_use]
    pub fn new(
        kind: TranscriptItemKind,
        title: impl Into<String>,
        body: impl Into<String>,
        streaming: bool,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            body: body.into(),
            streaming,
            metadata_id: None,
            reasoning_visibility: ReasoningVisibility::Auto,
            tool_output_expanded: false,
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
}

impl TranscriptState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            items: Vec::new(),
            scroll_offset: 0,
            limit: limit.max(1),
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
            UiEventPayload::UserMessage { message } => self.push(TranscriptItem::new(
                TranscriptItemKind::User,
                "user",
                message,
                false,
            )),
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
                self.close_streams_before_tool();
                self.update_tool_with_detail(&call_id, "tool running", "observation", &observation);
            }
            UiEventPayload::ToolResult {
                call_id,
                ok,
                summary,
                elapsed_ms,
            } => {
                self.close_streams_before_tool();
                let status = if ok { "ok" } else { "failed" };
                let detail = tool_result_detail(&summary, elapsed_ms);
                if !self.update_tool_with_detail(
                    &call_id,
                    &format!("tool {status}"),
                    "result",
                    &detail,
                ) {
                    self.push(TranscriptItem::new(
                        TranscriptItemKind::ToolResult,
                        "tool result",
                        format!("{status} {call_id}: {detail}"),
                        false,
                    ));
                }
            }
            UiEventPayload::ApprovalRequested(_) => {
                // Pending approvals live in the modal + footer indicator only.
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
        self.close_or_remove_reasoning_placeholder();
        if let Some(item) = self
            .items
            .last_mut()
            .filter(|item| item.kind == TranscriptItemKind::Model && item.streaming)
        {
            if item.body == MODEL_PENDING_PLACEHOLDER {
                item.body.clear();
            }
            item.body.push_str(delta);
            return;
        }

        // Empty deltas must not open a ghost model item — that would paint as a
        // blank row and inflate inter-item spacing once a real message arrives.
        if delta.is_empty() {
            return;
        }

        self.close_model_stream_without_final();
        self.push(TranscriptItem::streaming_model(delta));
    }

    fn append_reasoning_delta(&mut self, delta: &str) {
        self.remove_pending_model_placeholder();
        // Drop a closed empty model that may have been left behind after a tool
        // boundary; it would only contribute blank spacing in the transcript.
        self.remove_empty_trailing_model();
        if let Some(item) = self
            .items
            .last_mut()
            .filter(|item| item.kind == TranscriptItemKind::Reasoning && item.streaming)
        {
            if item.body == REASONING_PENDING_PLACEHOLDER {
                item.body.clear();
            }
            item.body.push_str(delta);
            return;
        }

        if delta.is_empty() {
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
        let title = "tool".to_string();
        // Keep the body free of call-id / policy / sandbox noise. The head is
        // just the friendly tool name + target; result lines come later.
        let body = tool_call_body(&tool_name, detail);
        if !self.update_tool_by_id(&call_id, title, body, status == "running") {
            self.push(TranscriptItem::tool_call(
                call_id, tool_name, status, detail,
            ));
        }
    }

    fn update_tool_by_id(
        &mut self,
        call_id: &str,
        title: String,
        body: String,
        streaming: bool,
    ) -> bool {
        let Some(item) = self.items.iter_mut().rev().find(|item| {
            matches!(
                item.kind,
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
            ) && item.metadata_id.as_deref() == Some(call_id)
        }) else {
            return false;
        };

        item.kind = if streaming {
            TranscriptItemKind::ToolCall
        } else {
            TranscriptItemKind::ToolResult
        };
        item.title = title;
        // Preserve an existing head (friendly name + target) when the new body
        // is just a status/policy blob with no useful target.
        if !body.trim().is_empty() {
            item.body = body;
        }
        item.streaming = streaming;
        true
    }

    fn update_tool_with_detail(
        &mut self,
        call_id: &str,
        title: &str,
        _label: &str,
        detail: &str,
    ) -> bool {
        let Some(item) = self.items.iter_mut().rev().find(|item| {
            matches!(
                item.kind,
                TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult
            ) && item.metadata_id.as_deref() == Some(call_id)
        }) else {
            return false;
        };

        // Keep the original head (friendly name + target) on the first line and put
        // observation/result on following lines so render can show:
        //   [tool] target
        //     ⎿ result…
        let prefix = item.body.split_once('\n').map_or_else(
            || {
                item.body
                    .split_once(" - ")
                    .map_or_else(|| item.body.clone(), |(prefix, _)| prefix.to_string())
            },
            |(prefix, _)| prefix.to_string(),
        );
        let streaming = title.ends_with("running");
        item.kind = if streaming {
            TranscriptItemKind::ToolCall
        } else {
            TranscriptItemKind::ToolResult
        };
        // Keep a stable, non-noisy title; visual status is the streaming flag
        // and result body, not "tool ok"/"tool failed" chrome.
        item.title = "tool".to_string();
        let cleaned = clean_tool_result_detail(detail);
        item.body = if cleaned.is_empty() {
            prefix
        } else {
            format!("{prefix}\n{cleaned}")
        };
        item.streaming = streaming;
        true
    }

    fn close_model_stream(&mut self, message: String) {
        self.close_or_remove_reasoning_placeholder();
        let body = if message.trim().is_empty() {
            "done".to_string()
        } else {
            message
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
        self.remove_pending_reasoning_placeholder();
        self.remove_pending_model_placeholder();
        self.close_reasoning_stream();
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
        if self.remove_pending_reasoning_placeholder() {
            return;
        }
        self.close_reasoning_stream();
    }

    fn close_reasoning_stream(&mut self) {
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Reasoning && item.streaming)
        {
            item.streaming = false;
        }
    }

    fn remove_pending_reasoning_placeholder(&mut self) -> bool {
        if self.items.last().is_some_and(|item| {
            item.kind == TranscriptItemKind::Reasoning
                && item.streaming
                && item.body == REASONING_PENDING_PLACEHOLDER
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

    // Prefer a compact path / command / message from common JSON shapes.
    if let Some(path) = json_string_field(without_elapsed, "path") {
        if let Some(bytes) = json_number_field(without_elapsed, "bytes") {
            return format!("{path} ({bytes} bytes)");
        }
        return path;
    }
    if let Some(message) = json_string_field(without_elapsed, "message") {
        return message;
    }
    if let Some(stdout) = json_string_field(without_elapsed, "stdout") {
        return stdout;
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

    // If it still looks like raw JSON, try to pretty-collapse to a short line.
    if stripped.starts_with('{') {
        if let Some(path) = json_string_field(stripped, "path") {
            return path;
        }
    }
    stripped.to_string()
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
