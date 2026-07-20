use agent_ui_bridge::{UiDiagnosticLevel, UiEventPayload, UiJsonFrame};

const DEFAULT_TRACE_LIMIT: usize = 300;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TraceEntryKind {
    Runtime,
    Tool,
    Task,
    Checkpoint,
    Diagnostic,
    External,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracePanelEntry {
    pub kind: TraceEntryKind,
    pub title: String,
    pub summary: String,
}

impl TracePanelEntry {
    #[must_use]
    pub fn new(kind: TraceEntryKind, title: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            summary: summary.into(),
        }
    }

    #[must_use]
    pub fn render_line(&self) -> String {
        format!("{}: {}", self.title, self.summary)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TraceFilter {
    #[default]
    All,
    Runtime,
    Tools,
    Tasks,
    Checkpoints,
    Diagnostics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracePanelState {
    pub entries: Vec<TracePanelEntry>,
    pub filter: TraceFilter,
    limit: usize,
}

impl TracePanelState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            filter: TraceFilter::All,
            limit: limit.max(1),
        }
    }

        pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::Event(event) => self.reduce_event_payload(&event.payload),
            UiJsonFrame::ConfigSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "config",
                format!(
                    "{}/{}",
                    snapshot.selected_provider, snapshot.selected_model
                ),
            )),
            UiJsonFrame::TerminalSessionSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "terminal-sessions",
                format!("{} sessions", snapshot.sessions.len()),
            )),
            UiJsonFrame::ApprovalQueueSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "approvals",
                format!("{} pending", snapshot.pending_count),
            )),
            UiJsonFrame::SlashCommandCatalogSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "slash",
                format!("{} commands", snapshot.commands.len()),
            )),
            UiJsonFrame::TerminalStatusSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "status",
                format!("{}/{}", snapshot.provider, snapshot.model),
            )),
            UiJsonFrame::EditDiffSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "edit-diff",
                snapshot.path.clone(),
            )),
            UiJsonFrame::ModelSelection { provider, model } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "model",
                format!("{provider}/{model}"),
            )),
            UiJsonFrame::ModelInvocationAttempts(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "model-attempts",
                snapshot.status.clone(),
            )),
            UiJsonFrame::ConfigSummary {
                selected_provider,
                selected_model,
                ..
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "config",
                format!("{selected_provider}/{selected_model}"),
            )),
            UiJsonFrame::ConfigDiagnostic {
                severity,
                code,
                message,
                ..
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "config-diagnostic",
                format!("{severity} {code}: {message}"),
            )),
            UiJsonFrame::PlanTodoSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "plan",
                format!("{} items", snapshot.snapshot.items.len()),
            )),
            UiJsonFrame::ProjectContextSnapshot(snapshot) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "project-context",
                snapshot.workspace_root_display.clone(),
            )),
            UiJsonFrame::ProjectContextDetail(detail) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "project-context-detail",
                detail.kind_label().to_string(),
            )),
            UiJsonFrame::Done { event_count } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "done",
                format!("{event_count} events"),
            )),
            UiJsonFrame::Error { code, message, .. } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                code.clone(),
                message.clone(),
            )),
            UiJsonFrame::Admin(admin) => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "admin",
                admin.kind.clone(),
            )),
        }
    }


    #[must_use]
    pub fn visible_entries(&self) -> Vec<&TracePanelEntry> {
        self.entries
            .iter()
            .filter(|entry| self.filter_allows(entry))
            .collect()
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let visible = self.visible_entries();
        if visible.is_empty() {
            return "No trace events".to_string();
        }

        visible
            .iter()
            .map(|entry| entry.render_line())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn reduce_event_payload(&mut self, payload: &UiEventPayload) {
        match payload {
            UiEventPayload::Progress { message } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "progress",
                message.clone(),
            )),
            UiEventPayload::TurnState { state, reason } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "turn",
                format!("{state:?}: {reason}"),
            )),
            UiEventPayload::ToolCall { call_id, tool_name, .. } => self.push(TracePanelEntry::new(
                TraceEntryKind::Tool,
                "tool call",
                format!("{tool_name} ({call_id})"),
            )),
            UiEventPayload::ToolResult {
                call_id,
                ok,
                summary,
                ..
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::Tool,
                "tool result",
                format!("{} {call_id}: {summary}", if *ok { "ok" } else { "failed" }),
            )),
            UiEventPayload::Checkpoint {
                checkpoint_id,
                label,
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::Checkpoint,
                "checkpoint",
                format!("{checkpoint_id}: {label}"),
            )),
            UiEventPayload::Diagnostic {
                level,
                code,
                message,
                recoverable,
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::Diagnostic,
                code.clone(),
                format!(
                    "{}: {message}{}",
                    diagnostic_level_name(*level),
                    if *recoverable { " (recoverable)" } else { "" }
                ),
            )),
            UiEventPayload::ExternalTrace {
                source_kind,
                source_id,
                summary,
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::External,
                source_kind.clone(),
                format!("{source_id}: {summary}"),
            )),
            UiEventPayload::TaskNotification(notification) => self.push(TracePanelEntry::new(
                TraceEntryKind::Task,
                format!("task {}", notification.kind),
                format!("{}: {}", notification.task_id, notification.summary),
            )),
            UiEventPayload::Permission {
                request_id,
                decision,
                reason,
            } => self.push(TracePanelEntry::new(
                TraceEntryKind::Runtime,
                "permission",
                format!("{request_id} {decision}: {reason}"),
            )),
            _ => {}
        }
    }

    fn push(&mut self, entry: TracePanelEntry) {
        self.entries.push(entry);
        let overflow = self.entries.len().saturating_sub(self.limit);
        if overflow > 0 {
            self.entries.drain(0..overflow);
        }
    }

    fn filter_allows(&self, entry: &TracePanelEntry) -> bool {
        if entry.kind == TraceEntryKind::Diagnostic {
            return true;
        }

        match self.filter {
            TraceFilter::All => true,
            TraceFilter::Runtime => entry.kind == TraceEntryKind::Runtime,
            TraceFilter::Tools => entry.kind == TraceEntryKind::Tool,
            TraceFilter::Tasks => entry.kind == TraceEntryKind::Task,
            TraceFilter::Checkpoints => entry.kind == TraceEntryKind::Checkpoint,
            TraceFilter::Diagnostics => false,
        }
    }
}

impl Default for TracePanelState {
    fn default() -> Self {
        Self::with_limit(DEFAULT_TRACE_LIMIT)
    }
}

fn diagnostic_level_name(level: UiDiagnosticLevel) -> &'static str {
    match level {
        UiDiagnosticLevel::Warning => "warning",
        UiDiagnosticLevel::Error => "error",
    }
}
