use agent_types::{
    ApprovalRequestId, ClientKind, ConversationId, CorrelationId, SchemaVersion, WorkspaceId,
};
use agent_ui_bridge::{
    UiApprovalDecision, UiBridgeError, UiClientInfo, UiCommand, UiCommandEnvelope,
    validate_ui_command,
};

use crate::{SlashCommand, SlashCommandError};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComposerState {
    pub text: String,
    pub multiline: bool,
    pub history: Vec<String>,
    history_cursor: Option<usize>,
}

impl ComposerState {
    pub fn reduce(&mut self, action: ComposerAction) -> TuiEffect {
        match action {
            ComposerAction::InsertText(text) => {
                self.history_cursor = None;
                self.text.push_str(&text);
                TuiEffect::None
            }
            ComposerAction::InsertChar(character) => {
                self.history_cursor = None;
                self.text.push(character);
                TuiEffect::None
            }
            ComposerAction::Backspace => {
                self.history_cursor = None;
                self.text.pop();
                TuiEffect::None
            }
            ComposerAction::InsertNewline => {
                if self.multiline {
                    self.history_cursor = None;
                    self.text.push('\n');
                }
                TuiEffect::None
            }
            ComposerAction::ToggleMultiline => {
                self.multiline = !self.multiline;
                TuiEffect::None
            }
            ComposerAction::Submit => self.submit(),
            ComposerAction::HistoryPrevious => self.history_previous(),
            ComposerAction::HistoryNext => self.history_next(),
            ComposerAction::Clear => {
                self.text.clear();
                self.history_cursor = None;
                TuiEffect::None
            }
            ComposerAction::Escape => {
                self.history_cursor = None;
                TuiEffect::ComposerBlurred
            }
        }
    }

    fn submit(&mut self) -> TuiEffect {
        let prompt = self.text.trim();
        if prompt.is_empty() {
            return TuiEffect::LocalDiagnostic("prompt cannot be empty".to_string());
        }

        if prompt.starts_with('/') {
            return match SlashCommand::parse(prompt) {
                Ok(command) => {
                    self.text.clear();
                    self.history_cursor = None;
                    TuiEffect::from_slash_command(command)
                }
                Err(error) => TuiEffect::from_slash_error(error),
            };
        }

        let prompt = prompt.to_string();
        self.history.push(prompt.clone());
        self.text.clear();
        self.history_cursor = None;
        TuiEffect::SubmitPrompt(prompt)
    }

    fn history_previous(&mut self) -> TuiEffect {
        if self.history.is_empty() {
            return TuiEffect::None;
        }

        let next_index = match self.history_cursor {
            Some(0) => 0,
            Some(index) => index.saturating_sub(1),
            None => self.history.len() - 1,
        };

        self.history_cursor = Some(next_index);
        self.text = self.history[next_index].clone();
        TuiEffect::None
    }

    fn history_next(&mut self) -> TuiEffect {
        let Some(index) = self.history_cursor else {
            return TuiEffect::None;
        };

        if index + 1 >= self.history.len() {
            self.history_cursor = None;
            self.text.clear();
        } else {
            let next_index = index + 1;
            self.history_cursor = Some(next_index);
            self.text = self.history[next_index].clone();
        }

        TuiEffect::None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComposerAction {
    InsertText(String),
    InsertChar(char),
    Backspace,
    InsertNewline,
    ToggleMultiline,
    Submit,
    HistoryPrevious,
    HistoryNext,
    Clear,
    Escape,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TuiEffect {
    None,
    SubmitPrompt(String),
    RunSlashCommand(SlashCommand),
    LocalDiagnostic(String),
    ComposerBlurred,
    ResolveApproval {
        request_id: ApprovalRequestId,
        decision: UiApprovalDecision,
        reason: String,
        lease_duration_seconds: Option<u64>,
    },
    ApprovalAborted(String),
    SwitchSession {
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
    },
}

impl TuiEffect {
    #[must_use]
    pub const fn from_slash_command(command: SlashCommand) -> Self {
        Self::RunSlashCommand(command)
    }

    #[must_use]
    pub fn from_slash_error(error: SlashCommandError) -> Self {
        Self::LocalDiagnostic(error.to_string())
    }

    pub fn to_ui_command(
        &self,
        context: &TuiBridgeContext,
    ) -> Result<Option<UiCommandEnvelope>, UiBridgeError> {
        let command = match self {
            Self::SubmitPrompt(prompt) => UiCommand::StartTurn {
                workspace_id: context.workspace_id,
                conversation_id: context.conversation_id,
                prompt: prompt.clone(),
            },
            Self::ResolveApproval {
                request_id,
                decision,
                reason,
                lease_duration_seconds,
            } => UiCommand::ResolveApproval {
                request_id: *request_id,
                decision: *decision,
                reason: reason.clone(),
                lease_duration_seconds: *lease_duration_seconds,
            },
            Self::SwitchSession {
                workspace_id,
                conversation_id,
            } => UiCommand::SwitchSession {
                workspace_id: *workspace_id,
                conversation_id: *conversation_id,
            },
            _ => return Ok(None),
        };

        let envelope = UiCommandEnvelope {
            schema_version: SchemaVersion::current(),
            command_id: CorrelationId::new(),
            client: UiClientInfo {
                kind: context.client_kind.clone(),
                version: context.client_version.clone(),
            },
            command,
        };

        validate_ui_command(&envelope)?;
        Ok(Some(envelope))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiBridgeContext {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub client_kind: ClientKind,
    pub client_version: String,
}

impl TuiBridgeContext {
    pub fn new(
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
        client_kind: ClientKind,
        client_version: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let client_version = client_version.into();
        if client_version.trim().is_empty() {
            return Err(UiBridgeError::Validation(
                "tui bridge client version cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            workspace_id,
            conversation_id,
            client_kind,
            client_version,
        })
    }
}
