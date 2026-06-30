use agent_ui_bridge::{UiDiagnosticLevel, UiEventPayload, UiJsonFrame};

const DEFAULT_TRANSCRIPT_LIMIT: usize = 500;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TranscriptItemKind {
    User,
    Model,
    ToolCall,
    ToolResult,
    Approval,
    Diagnostic,
    Progress,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptItem {
    pub kind: TranscriptItemKind,
    pub title: String,
    pub body: String,
    pub streaming: bool,
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
    pub fn render_line(&self) -> String {
        let state = if self.streaming { " streaming" } else { "" };
        format!("{}{}: {}", self.title, state, self.body)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptState {
    pub items: Vec<TranscriptItem>,
    limit: usize,
}

impl TranscriptState {
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            items: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn push(&mut self, item: TranscriptItem) {
        self.items.push(item);
        self.enforce_limit();
    }

    pub fn reduce_frame(&mut self, frame: UiJsonFrame) {
        match frame {
            UiJsonFrame::Event(event) => self.reduce_event_payload(event.payload),
            UiJsonFrame::ConfigDiagnostic {
                severity,
                code,
                message,
                ..
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Diagnostic,
                code,
                format!("{severity}: {message}"),
                false,
            )),
            UiJsonFrame::Error {
                code,
                message,
                recoverable,
            } => self.push(TranscriptItem::new(
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
            )),
            UiJsonFrame::ModelSelection { provider, model } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "model",
                format!("{provider}/{model}"),
                false,
            )),
            UiJsonFrame::ConfigSummary {
                selected_provider,
                selected_model,
                ..
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "config",
                format!("{selected_provider}/{selected_model}"),
                false,
            )),
            UiJsonFrame::ConfigSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "config",
                format!("{}/{}", snapshot.selected_provider, snapshot.selected_model),
                false,
            )),
            UiJsonFrame::RepoSessionSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "repo-session",
                format!(
                    "{} git={} provider={}/{} diagnostics={}",
                    snapshot.workspace_root_display,
                    snapshot.git_state,
                    snapshot.selected_provider,
                    snapshot.selected_model,
                    snapshot.diagnostic_count
                ),
                false,
            )),
            UiJsonFrame::EditDiffSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "edit-diff",
                format!(
                    "{} state={} +{} -{} files={}",
                    snapshot.path,
                    snapshot.diff_state,
                    snapshot.added_line_count,
                    snapshot.removed_line_count,
                    snapshot.changed_file_count
                ),
                false,
            )),
            UiJsonFrame::CommandTestSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "command-test",
                format!(
                    "{} status={} classification={} exit={}",
                    snapshot.command,
                    snapshot.status,
                    snapshot.classification,
                    snapshot
                        .exit_code
                        .map_or_else(|| "none".to_string(), |code| code.to_string())
                ),
                false,
            )),
            UiJsonFrame::FinalReviewSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "final-review",
                format!(
                    "files={} commands={} risks={} next={} replay={}",
                    snapshot.changed_files.len(),
                    snapshot.command_results.len(),
                    snapshot.unresolved_risks.len(),
                    snapshot.next_actions.len(),
                    snapshot.replay_reconstructed
                ),
                false,
            )),
            UiJsonFrame::PromptSummary(summary) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "prompt",
                format!(
                    "{} v{} ({} messages)",
                    summary.display_name, summary.profile_version, summary.message_count
                ),
                false,
            )),
            UiJsonFrame::ProjectContextSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "project-context",
                format!(
                    "{} files={} dirs={} omitted={} diagnostics={}",
                    snapshot.workspace_root_display,
                    snapshot.file_count,
                    snapshot.directory_count,
                    snapshot.omitted_count,
                    snapshot.diagnostic_count
                ),
                false,
            )),
            UiJsonFrame::ExtensionSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "extensions",
                format!(
                    "{} installed, {} reviews",
                    snapshot.extensions.len(),
                    snapshot.capability_reviews.len()
                ),
                false,
            )),
            UiJsonFrame::TaskSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "tasks",
                format!(
                    "{} tasks, {} errors",
                    snapshot.tasks.len(),
                    snapshot.errors.len()
                ),
                false,
            )),
            UiJsonFrame::TaskReviewSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "task-reviews",
                format!(
                    "{} reviews, {} errors",
                    snapshot.reviews.len(),
                    snapshot.errors.len()
                ),
                false,
            )),
            UiJsonFrame::StartupMigrationSnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "startup",
                format!(
                    "runtime_allowed={} steps={} applied={} diagnostics={}",
                    snapshot.runtime_allowed,
                    snapshot.step_count,
                    snapshot.applied_count,
                    snapshot.diagnostic_count
                ),
                false,
            )),
            UiJsonFrame::ObservabilitySnapshot(snapshot) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "observability",
                format!(
                    "events={} trace={} audit={} telemetry={} replayable={} diagnostics={}",
                    snapshot.event_count,
                    snapshot.trace_count,
                    snapshot.audit_count,
                    snapshot.telemetry_count,
                    snapshot.replayable,
                    snapshot.diagnostics.len()
                ),
                false,
            )),
            UiJsonFrame::Done { event_count } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "done",
                format!("{event_count} events"),
                false,
            )),
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
            UiEventPayload::ModelDelta { delta } => self.append_model_delta(&delta),
            UiEventPayload::FinalResponse { message } => self.close_model_stream(message),
            UiEventPayload::ToolCall { call_id, tool_name } => self.push(TranscriptItem::new(
                TranscriptItemKind::ToolCall,
                "tool call",
                format!("{tool_name} ({call_id})"),
                false,
            )),
            UiEventPayload::ToolResult {
                call_id,
                ok,
                summary,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::ToolResult,
                "tool result",
                format!("{} {call_id}: {summary}", if ok { "ok" } else { "failed" }),
                false,
            )),
            UiEventPayload::ApprovalRequested(request) => self.push(TranscriptItem::new(
                TranscriptItemKind::Approval,
                "approval",
                format!(
                    "{} {} {}: {}{}",
                    request.action,
                    request.resource_kind,
                    request.resource,
                    request.preview.summary,
                    if request.preview.redacted {
                        " (redacted)"
                    } else {
                        ""
                    }
                ),
                false,
            )),
            UiEventPayload::ApprovalResolved {
                request_id,
                status,
                decision,
                reason,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Approval,
                "approval",
                format!(
                    "{request_id} {status} {}{}",
                    decision.unwrap_or_else(|| "no_decision".to_string()),
                    reason
                        .map(|reason| format!(": {reason}"))
                        .unwrap_or_default()
                ),
                false,
            )),
            UiEventPayload::Diagnostic {
                level,
                code,
                message,
                recoverable,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Diagnostic,
                code,
                format!(
                    "{}: {message}{}",
                    diagnostic_level_name(level),
                    if recoverable { " (recoverable)" } else { "" }
                ),
                false,
            )),
            UiEventPayload::Progress { message } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "progress",
                message,
                false,
            )),
            UiEventPayload::TurnState { state, reason } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "turn",
                format!("{state:?}: {reason}"),
                false,
            )),
            UiEventPayload::Permission {
                request_id,
                decision,
                reason,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "permission",
                format!("{request_id} {decision}: {reason}"),
                false,
            )),
            UiEventPayload::Checkpoint {
                checkpoint_id,
                label,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "checkpoint",
                format!("{checkpoint_id}: {label}"),
                false,
            )),
            UiEventPayload::PromptSummary(summary) => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "prompt",
                format!(
                    "{} v{} ({} messages)",
                    summary.display_name, summary.profile_version, summary.message_count
                ),
                false,
            )),
            UiEventPayload::TaskNotification(notification) => {
                self.push(TranscriptItem::new(
                    TranscriptItemKind::Progress,
                    &notification.kind,
                    &notification.summary,
                    false,
                ));
            }
            UiEventPayload::ExternalTrace {
                source_kind,
                source_id,
                summary,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                source_kind,
                format!("{source_id}: {summary}"),
                false,
            )),
            UiEventPayload::Workspace {
                root_path,
                trust_level,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "workspace",
                format!("{root_path} ({trust_level})"),
                false,
            )),
            UiEventPayload::Session {
                client_kind,
                client_version,
            } => self.push(TranscriptItem::new(
                TranscriptItemKind::Progress,
                "session",
                format!("{client_kind} {client_version}"),
                false,
            )),
        }
    }

    fn append_model_delta(&mut self, delta: &str) {
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Model && item.streaming)
        {
            item.body.push_str(delta);
            return;
        }

        self.push(TranscriptItem::streaming_model(delta));
    }

    fn close_model_stream(&mut self, message: String) {
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.kind == TranscriptItemKind::Model && item.streaming)
        {
            item.body = message;
            item.streaming = false;
            return;
        }

        self.push(TranscriptItem::completed_model(message));
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

fn diagnostic_level_name(level: UiDiagnosticLevel) -> &'static str {
    match level {
        UiDiagnosticLevel::Warning => "warning",
        UiDiagnosticLevel::Error => "error",
    }
}
