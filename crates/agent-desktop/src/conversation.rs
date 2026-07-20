use agent_types::{ConversationId, CorrelationId, Timestamp, WorkspaceId};
use agent_ui_bridge::{UiEventPayload, UiJsonFrame};
use serde::{Deserialize, Serialize};

use crate::DesktopError;

const MAX_CONVERSATION_TEXT_CHARS: usize = 240;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopConversationItem {
    pub conversation_id: ConversationId,
    pub title: String,
    pub last_activity_at: Timestamp,
    pub unread_count: u32,
}

impl DesktopConversationItem {
    pub fn new(
        conversation_id: ConversationId,
        title: impl Into<String>,
        last_activity_at: Timestamp,
        unread_count: u32,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            conversation_id,
            title: validate_non_empty("conversation title", title.into())?,
            last_activity_at,
            unread_count,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("conversation title", self.title.clone())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopConversationPanel {
    pub workspace_id: WorkspaceId,
    pub active_conversation_id: Option<ConversationId>,
    pub conversations: Vec<DesktopConversationItem>,
    pub transcript: Vec<DesktopTranscriptEntry>,
    pub current_submission: Option<DesktopTurnSubmission>,
    pub diagnostics: Vec<DesktopConversationDiagnostic>,
}

impl DesktopConversationPanel {
    pub fn new(
        workspace_id: WorkspaceId,
        active_conversation_id: Option<ConversationId>,
        conversations: Vec<DesktopConversationItem>,
    ) -> Result<Self, DesktopError> {
        for conversation in &conversations {
            conversation.validate()?;
        }
        Ok(Self {
            workspace_id,
            active_conversation_id,
            conversations,
            transcript: Vec::new(),
            current_submission: None,
            diagnostics: Vec::new(),
        })
    }

    pub fn record_submission(
        &mut self,
        submission: DesktopTurnSubmission,
    ) -> Result<(), DesktopError> {
        if Some(submission.conversation_id) != self.active_conversation_id {
            return Err(DesktopError::InvalidSession(
                "turn submission conversation must match active conversation".to_string(),
            ));
        }
        self.current_submission = Some(submission);
        Ok(())
    }

    pub fn apply_ack(&mut self, ack: DesktopTurnCommandAck) -> Result<(), DesktopError> {
        let Some(submission) = self.current_submission.as_mut() else {
            return Err(DesktopError::InvalidSession(
                "cannot apply command ack without current submission".to_string(),
            ));
        };
        submission.apply_ack(ack)
    }

    pub fn apply_frame(&mut self, frame: &UiJsonFrame) -> Result<(), DesktopError> {
        match conversation_update_from_frame(frame)? {
            DesktopConversationFrameUpdate::Transcript(entry) => {
                self.transcript.push(entry);
            }
            DesktopConversationFrameUpdate::Done { .. } => {
                if let Some(submission) = self.current_submission.as_mut() {
                    submission.complete();
                }
            }
            DesktopConversationFrameUpdate::Error(error) => {
                if let Some(submission) = self.current_submission.as_mut() {
                    submission.apply_error(error.clone());
                }
                self.diagnostics
                    .push(DesktopConversationDiagnostic::from_error(error));
            }
            DesktopConversationFrameUpdate::Ignored => {}
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTranscriptEntry {
    pub id: String,
    pub conversation_id: Option<ConversationId>,
    pub role: DesktopTranscriptRole,
    pub content: String,
    pub source_event_id: Option<String>,
    pub truncated: bool,
}

impl DesktopTranscriptEntry {
    pub fn new(
        id: impl Into<String>,
        conversation_id: Option<ConversationId>,
        role: DesktopTranscriptRole,
        content: impl Into<String>,
        source_event_id: Option<String>,
    ) -> Result<Self, DesktopError> {
        let (id, id_truncated) = bounded_required_text("transcript entry id", id)?;
        let (content, content_truncated) = bounded_required_text("transcript content", content)?;
        Ok(Self {
            id,
            conversation_id,
            role,
            content,
            source_event_id: source_event_id
                .map(bounded_optional_text)
                .transpose()?
                .flatten(),
            truncated: id_truncated || content_truncated,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopTranscriptRole {
    User,
    Assistant,
    Diagnostic,
    Progress,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTurnSubmission {
    pub command_id: CorrelationId,
    pub conversation_id: ConversationId,
    pub prompt_preview: String,
    pub status: DesktopTurnSubmissionStatus,
    pub ack: Option<DesktopTurnCommandAck>,
    pub error: Option<DesktopBridgeErrorNotice>,
    pub truncated: bool,
}

impl DesktopTurnSubmission {
    pub fn submitting(
        command_id: CorrelationId,
        conversation_id: ConversationId,
        prompt: impl Into<String>,
    ) -> Result<Self, DesktopError> {
        let (prompt_preview, truncated) = bounded_required_text("turn prompt", prompt)?;
        Ok(Self {
            command_id,
            conversation_id,
            prompt_preview,
            status: DesktopTurnSubmissionStatus::Submitting,
            ack: None,
            error: None,
            truncated,
        })
    }

    pub fn apply_ack(&mut self, ack: DesktopTurnCommandAck) -> Result<(), DesktopError> {
        if ack.command_id != self.command_id {
            return Err(DesktopError::InvalidSession(
                "command ack id must match current submission".to_string(),
            ));
        }
        self.status = if ack.accepted {
            DesktopTurnSubmissionStatus::Accepted
        } else {
            DesktopTurnSubmissionStatus::Rejected
        };
        self.ack = Some(ack);
        Ok(())
    }

    pub fn complete(&mut self) {
        self.status = DesktopTurnSubmissionStatus::Completed;
    }

    pub fn apply_error(&mut self, error: DesktopBridgeErrorNotice) {
        self.status = DesktopTurnSubmissionStatus::Rejected;
        self.error = Some(error);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopTurnSubmissionStatus {
    Draft,
    Submitting,
    Accepted,
    Rejected,
    DrainingFrames,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTurnCommandAck {
    pub command_id: CorrelationId,
    pub accepted: bool,
    pub pending_frame_count: u32,
}

impl DesktopTurnCommandAck {
    #[must_use]
    pub const fn new(command_id: CorrelationId, accepted: bool, pending_frame_count: u32) -> Self {
        Self {
            command_id,
            accepted,
            pending_frame_count,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopBridgeErrorNotice {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub truncated: bool,
}

impl DesktopBridgeErrorNotice {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        recoverable: bool,
    ) -> Result<Self, DesktopError> {
        let (code, code_truncated) = bounded_required_text("bridge error code", code)?;
        let (message, message_truncated) = bounded_required_text("bridge error message", message)?;
        Ok(Self {
            code,
            message,
            recoverable,
            truncated: code_truncated || message_truncated,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopConversationDiagnostic {
    pub severity: DesktopConversationDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub truncated: bool,
}

impl DesktopConversationDiagnostic {
    fn from_error(error: DesktopBridgeErrorNotice) -> Self {
        Self {
            severity: if error.recoverable {
                DesktopConversationDiagnosticSeverity::Warning
            } else {
                DesktopConversationDiagnosticSeverity::Error
            },
            code: error.code,
            message: error.message,
            truncated: error.truncated,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopConversationDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DesktopConversationFrameUpdate {
    Transcript(DesktopTranscriptEntry),
    Done { event_count: u64 },
    Error(DesktopBridgeErrorNotice),
    Ignored,
}

pub fn conversation_update_from_frame(
    frame: &UiJsonFrame,
) -> Result<DesktopConversationFrameUpdate, DesktopError> {
    match frame {
        UiJsonFrame::Event(envelope) => {
            let Some((role, content)) = transcript_payload(&envelope.payload) else {
                return Ok(DesktopConversationFrameUpdate::Ignored);
            };
            Ok(DesktopConversationFrameUpdate::Transcript(
                DesktopTranscriptEntry::new(
                    envelope.event_id.to_string(),
                    Some(envelope.conversation_id),
                    role,
                    content,
                    Some(envelope.event_id.to_string()),
                )?,
            ))
        }
        UiJsonFrame::Done { event_count } => Ok(DesktopConversationFrameUpdate::Done {
            event_count: *event_count,
        }),
        UiJsonFrame::Error {
            code,
            message,
            recoverable,
        } => Ok(DesktopConversationFrameUpdate::Error(
            DesktopBridgeErrorNotice::new(code.clone(), message.clone(), *recoverable)?,
        )),
        _ => Ok(DesktopConversationFrameUpdate::Ignored),
    }
}

pub(crate) fn validate_non_empty(label: &str, value: String) -> Result<String, DesktopError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopError::InvalidSnapshot(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}

fn transcript_payload(payload: &UiEventPayload) -> Option<(DesktopTranscriptRole, String)> {
    match payload {
        UiEventPayload::UserMessage { message } => {
            Some((DesktopTranscriptRole::User, message.clone()))
        }
        UiEventPayload::ModelDelta { delta } => {
            Some((DesktopTranscriptRole::Assistant, delta.clone()))
        }
        UiEventPayload::ReasoningSummaryDelta { delta } => {
            Some((DesktopTranscriptRole::Progress, delta.clone()))
        }
        UiEventPayload::FinalResponse { message } => {
            Some((DesktopTranscriptRole::Assistant, message.clone()))
        }
        UiEventPayload::Diagnostic { message, .. } => {
            Some((DesktopTranscriptRole::Diagnostic, message.clone()))
        }
        UiEventPayload::Progress { message } => {
            Some((DesktopTranscriptRole::Progress, message.clone()))
        }
        _ => None,
    }
}

fn bounded_required_text(
    label: &str,
    value: impl Into<String>,
) -> Result<(String, bool), DesktopError> {
    let value = value.into();
    let value = value.trim();
    if value.is_empty() {
        return Err(DesktopError::InvalidSnapshot(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(bound_conversation_text(value))
}

fn bounded_optional_text(value: String) -> Result<Option<String>, DesktopError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(bound_conversation_text(value).0))
}

fn bound_conversation_text(value: &str) -> (String, bool) {
    let redacted = redact_secret_like_text(value);
    let redacted = redacted.trim();
    if redacted.chars().count() > MAX_CONVERSATION_TEXT_CHARS {
        (
            redacted.chars().take(MAX_CONVERSATION_TEXT_CHARS).collect(),
            true,
        )
    } else {
        (redacted.to_string(), false)
    }
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if is_secret_like(token) {
                "[redacted]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.starts_with("sk-")
}
