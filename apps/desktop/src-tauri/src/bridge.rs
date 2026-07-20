use std::collections::VecDeque;

use agent_core::{PlatformPathOverrides, PlatformPaths, TerminalSessionCatalog};
use agent_desktop::{
    DesktopConversationItem, DesktopSessionReadiness, DesktopSessionReadinessState,
};
use agent_storage::{EventStore, SqliteEventStore};
use agent_types::{
    ActorKind, ActorRef, ConversationId, CorrelationId, EventId, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, SessionId, Timestamp, TurnId, UserInputReceived, WorkspaceId,
};
use agent_ui_bridge::{
    UiCommand, UiCommandEnvelope, UiJsonFrame, normalize_runtime_event, validate_ui_command,
};
use serde::{Deserialize, Serialize};

use crate::bootstrap::DesktopRuntimeState;
use crate::commands::DesktopCommandError;
#[cfg(test)]
use crate::commands::redact_secret_like_text;

const DEFAULT_FRAME_CAPACITY: usize = 256;
const DESKTOP_EVENT_STORE_FILE: &str = "terminal-events.sqlite3";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopStartSessionRequest {
    pub workspace_path: Option<String>,
    pub client_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopSessionSnapshot {
    pub session_id: String,
    pub workspace_id: String,
    pub conversation_id: String,
    pub state: DesktopRuntimeState,
    pub pending_frame_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopCommandAck {
    pub command_id: String,
    pub accepted: bool,
    pub state: DesktopRuntimeState,
    pub pending_frame_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SessionIdentity {
    session_id: SessionId,
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
}

impl SessionIdentity {
    fn new() -> Self {
        Self {
            session_id: SessionId::new(),
            workspace_id: WorkspaceId::new(),
            conversation_id: ConversationId::new(),
        }
    }
}

pub struct DesktopRuntimeSession {
    identity: Option<SessionIdentity>,
    state: DesktopRuntimeState,
    frame_capacity: usize,
    frames: VecDeque<UiJsonFrame>,
    event_store: Option<SqliteEventStore>,
}

impl Default for DesktopRuntimeSession {
    fn default() -> Self {
        Self {
            identity: None,
            state: DesktopRuntimeState::NotStarted,
            frame_capacity: DEFAULT_FRAME_CAPACITY,
            frames: VecDeque::new(),
            event_store: None,
        }
    }
}

impl std::fmt::Debug for DesktopRuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DesktopRuntimeSession")
            .field("identity", &self.identity)
            .field("state", &self.state)
            .field("frame_capacity", &self.frame_capacity)
            .field("pending_frame_count", &self.frames.len())
            .field("event_store_ready", &self.event_store.is_some())
            .finish()
    }
}

impl DesktopRuntimeSession {
    #[must_use]
    pub fn new(frame_capacity: usize) -> Self {
        Self {
            identity: None,
            state: DesktopRuntimeState::NotStarted,
            frame_capacity,
            frames: VecDeque::new(),
            event_store: Some(
                SqliteEventStore::open_in_memory()
                    .expect("desktop in-memory event store initializes"),
            ),
        }
    }

    #[cfg(test)]
    #[must_use]
    pub fn new_for_tests(frame_capacity: usize) -> Self {
        Self::new(frame_capacity)
    }

    #[cfg(test)]
    #[must_use]
    pub fn new_with_event_store_for_tests(
        frame_capacity: usize,
        event_store: SqliteEventStore,
    ) -> Self {
        Self {
            identity: None,
            state: DesktopRuntimeState::NotStarted,
            frame_capacity,
            frames: VecDeque::new(),
            event_store: Some(event_store),
        }
    }

    pub fn start(
        &mut self,
        request: DesktopStartSessionRequest,
    ) -> Result<DesktopSessionSnapshot, DesktopCommandError> {
        let client_version = request.client_version.trim();
        if client_version.is_empty() {
            return Err(DesktopCommandError::new(
                "desktop.bridge.invalid_start",
                "client version cannot be empty",
            ));
        }

        if self.state == DesktopRuntimeState::Connected {
            return Ok(self.snapshot());
        }

        self.state = DesktopRuntimeState::Connecting;
        self.ensure_event_store()?;
        self.identity.get_or_insert_with(SessionIdentity::new);
        self.state = DesktopRuntimeState::Connected;
        self.enqueue_frame(UiJsonFrame::ConfigDiagnostic {
            severity: "info".to_string(),
            code: "desktop.bridge.session_started".to_string(),
            message: format!("desktop bridge connected for client {client_version}"),
            field_path: None,
            source: request.workspace_path.and_then(normalize_optional_string),
        });

        Ok(self.snapshot())
    }

    pub fn stop(&mut self) -> Result<DesktopSessionSnapshot, DesktopCommandError> {
        self.identity.get_or_insert_with(SessionIdentity::new);
        self.state = DesktopRuntimeState::Disconnected;
        self.frames.clear();
        Ok(self.snapshot())
    }

    pub fn send_ui_command(
        &mut self,
        envelope: UiCommandEnvelope,
    ) -> Result<DesktopCommandAck, DesktopCommandError> {
        if self.state != DesktopRuntimeState::Connected {
            return Err(DesktopCommandError::new(
                "desktop.bridge.disconnected",
                "desktop bridge session is not connected",
            ));
        }

        validate_ui_command(&envelope).map_err(|error| {
            DesktopCommandError::new("desktop.bridge.command_invalid", error.to_string())
        })?;

        let command_frames = match &envelope.command {
            UiCommand::SwitchSession {
                workspace_id,
                conversation_id,
            } => {
                let frames = self.replay_conversation_frames(*conversation_id)?;
                let identity = self.identity.get_or_insert_with(SessionIdentity::new);
                identity.workspace_id = *workspace_id;
                identity.conversation_id = *conversation_id;
                frames
            }
            UiCommand::StartTurn {
                workspace_id,
                conversation_id,
                prompt,
            } => {
                let event = desktop_user_input_event(
                    *workspace_id,
                    *conversation_id,
                    envelope.command_id,
                    prompt.clone(),
                );
                let stored = self
                    .event_store
                    .as_mut()
                    .ok_or_else(|| {
                        DesktopCommandError::new(
                            "desktop.bridge.event_store_unavailable",
                            "desktop event store is unavailable",
                        )
                    })?
                    .append(event)
                    .map_err(|error| {
                        DesktopCommandError::new(
                            "desktop.bridge.event_append_failed",
                            error.to_string(),
                        )
                    })?;
                let identity = self.identity.get_or_insert_with(SessionIdentity::new);
                identity.workspace_id = *workspace_id;
                identity.conversation_id = *conversation_id;
                vec![UiJsonFrame::Event(normalize_runtime_event(&stored.event))]
            }
            _ => Vec::new(),
        };

        let command_id = envelope.command_id.to_string();
        let event_count = u64::try_from(command_frames.len()).unwrap_or(u64::MAX);
        for frame in command_frames {
            self.enqueue_frame(frame);
        }
        self.enqueue_frame(UiJsonFrame::Done { event_count });

        Ok(DesktopCommandAck {
            command_id,
            accepted: true,
            state: self.state,
            pending_frame_count: self.frames.len(),
        })
    }

    pub fn drain_ui_frames(
        &mut self,
        max_count: usize,
    ) -> Result<Vec<UiJsonFrame>, DesktopCommandError> {
        let mut drained = Vec::new();
        for _ in 0..max_count {
            let Some(frame) = self.frames.pop_front() else {
                break;
            };
            drained.push(frame);
        }
        Ok(drained)
    }

    #[must_use]
    pub fn readiness(&self) -> DesktopSessionReadiness {
        let identity = self.identity.as_ref();
        DesktopSessionReadiness {
            state: match self.state {
                DesktopRuntimeState::NotStarted => DesktopSessionReadinessState::NotStarted,
                DesktopRuntimeState::Connecting => DesktopSessionReadinessState::Connecting,
                DesktopRuntimeState::Connected => DesktopSessionReadinessState::Connected,
                DesktopRuntimeState::Disconnected => DesktopSessionReadinessState::Disconnected,
                DesktopRuntimeState::Blocked => DesktopSessionReadinessState::Blocked,
            },
            session_id: identity.map(|identity| identity.session_id),
            conversation_id: identity.map(|identity| identity.conversation_id),
            pending_frame_count: usize_to_u32(self.frames.len()),
        }
    }

    pub fn persisted_conversations(
        &self,
    ) -> Result<Vec<DesktopConversationItem>, DesktopCommandError> {
        let Some(store) = self.event_store.as_ref() else {
            return Ok(Vec::new());
        };
        let mut catalog = TerminalSessionCatalog::default();
        catalog.hydrate_from_store(store).map_err(|error| {
            DesktopCommandError::new(
                "desktop.workspace_snapshot.history_failed",
                error.to_string(),
            )
        })?;
        let report = catalog.report().map_err(|error| {
            DesktopCommandError::new(
                "desktop.workspace_snapshot.history_invalid",
                error.to_string(),
            )
        })?;
        report
            .sessions
            .into_iter()
            .map(|session| {
                DesktopConversationItem::new(
                    session.conversation_id,
                    session.title,
                    session.last_event_at,
                    0,
                )
                .map_err(|error| {
                    DesktopCommandError::new(
                        "desktop.workspace_snapshot.conversation_invalid",
                        error.to_string(),
                    )
                })
            })
            .collect()
    }

    #[must_use]
    pub fn workspace_snapshot_identity(&self) -> (WorkspaceId, Option<ConversationId>) {
        self.identity.as_ref().map_or_else(
            || (nil_workspace_id(), None),
            |identity| (identity.workspace_id, Some(identity.conversation_id)),
        )
    }

    pub(crate) fn ensure_workspace_snapshot_ids(&mut self) -> (WorkspaceId, ConversationId) {
        let identity = self.identity.get_or_insert_with(SessionIdentity::new);
        (identity.workspace_id, identity.conversation_id)
    }

    #[cfg(test)]
    pub fn append_event_for_tests(
        &mut self,
        event: RuntimeEvent,
    ) -> Result<(), DesktopCommandError> {
        self.event_store
            .as_mut()
            .expect("test event store is initialized")
            .append(event)
            .map(|_| ())
            .map_err(|error| {
                DesktopCommandError::new("desktop.bridge.test_event_append", error.to_string())
            })
    }

    #[cfg(test)]
    pub fn clear_frames_for_tests(&mut self) {
        self.frames.clear();
    }

    #[cfg(test)]
    pub fn enqueue_diagnostic_for_tests(&mut self, message: impl Into<String>) {
        self.enqueue_diagnostic(message.into());
    }

    fn ensure_event_store(&mut self) -> Result<(), DesktopCommandError> {
        if self.event_store.is_some() {
            return Ok(());
        }
        let paths = PlatformPaths::current(PlatformPathOverrides::default()).map_err(|error| {
            DesktopCommandError::new("desktop.bridge.data_dir_failed", error.to_string())
        })?;
        let store = SqliteEventStore::open(paths.data_dir.join(DESKTOP_EVENT_STORE_FILE)).map_err(
            |error| {
                DesktopCommandError::new("desktop.bridge.event_store_failed", error.to_string())
            },
        )?;
        self.event_store = Some(store);
        Ok(())
    }

    fn replay_conversation_frames(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Vec<UiJsonFrame>, DesktopCommandError> {
        let Some(store) = self.event_store.as_ref() else {
            return Ok(Vec::new());
        };
        store
            .list_by_conversation(conversation_id)
            .map(|events| {
                events
                    .into_iter()
                    .map(|stored| UiJsonFrame::Event(normalize_runtime_event(&stored.event)))
                    .collect()
            })
            .map_err(|error| {
                DesktopCommandError::new("desktop.bridge.history_replay_failed", error.to_string())
            })
    }

    fn snapshot(&self) -> DesktopSessionSnapshot {
        let identity = self
            .identity
            .as_ref()
            .expect("session identity exists before snapshot");
        DesktopSessionSnapshot {
            session_id: identity.session_id.to_string(),
            workspace_id: identity.workspace_id.to_string(),
            conversation_id: identity.conversation_id.to_string(),
            state: self.state,
            pending_frame_count: self.frames.len(),
        }
    }

    #[cfg(test)]
    fn enqueue_diagnostic(&mut self, message: String) {
        self.enqueue_frame(UiJsonFrame::Error {
            code: "desktop.bridge.diagnostic".to_string(),
            message: redact_secret_like_text(&message),
            recoverable: true,
        });
    }

    fn enqueue_frame(&mut self, frame: UiJsonFrame) {
        if self.frame_capacity == 0 {
            return;
        }

        if self.frames.len() >= self.frame_capacity {
            self.frames.pop_front();
            self.frames.push_back(UiJsonFrame::Error {
                code: "desktop.bridge.frames_dropped".to_string(),
                message: "one or more desktop bridge frames were dropped".to_string(),
                recoverable: true,
            });
            return;
        }

        self.frames.push_back(frame);
    }
}

fn desktop_user_input_event(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    correlation_id: CorrelationId,
    message: String,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id,
        conversation_id,
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef {
            kind: ActorKind::User,
            id: "desktop-user".to_string(),
            display_name: Some("desktop user".to_string()),
        },
        correlation_id,
        causation_id: None,
        payload: RuntimeEventPayload::UserInputReceived(UserInputReceived { message }),
    }
}

fn normalize_optional_string(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn nil_workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000000"
        .parse()
        .expect("nil workspace id is valid")
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
