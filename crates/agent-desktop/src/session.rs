use agent_types::{SessionId, Timestamp, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::DesktopError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSessionState {
    Connected,
    Disconnected,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRoute {
    Dashboard,
    Conversation,
    Tasks,
    Trace,
    Settings,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopSession {
    pub session_id: SessionId,
    pub workspace_id: WorkspaceId,
    pub client_version: String,
    pub route: DesktopRoute,
    pub state: DesktopSessionState,
    pub connected_at: Timestamp,
    pub updated_at: Timestamp,
}

pub fn start_desktop_session(
    workspace_id: WorkspaceId,
    client_version: impl Into<String>,
    now: Timestamp,
) -> Result<DesktopSession, DesktopError> {
    let client_version = validate_client_version(client_version.into())?;
    Ok(DesktopSession {
        session_id: SessionId::new(),
        workspace_id,
        client_version,
        route: DesktopRoute::Dashboard,
        state: DesktopSessionState::Connected,
        connected_at: now,
        updated_at: now,
    })
}

pub fn switch_desktop_route(
    session: &DesktopSession,
    route: DesktopRoute,
    now: Timestamp,
) -> Result<DesktopSession, DesktopError> {
    if session.state != DesktopSessionState::Connected {
        return Err(DesktopError::InvalidSession(
            "disconnected desktop sessions cannot switch routes".to_string(),
        ));
    }

    let mut next = session.clone();
    next.route = route;
    next.updated_at = now;
    Ok(next)
}

pub fn disconnect_desktop_session(
    session: &DesktopSession,
    now: Timestamp,
) -> Result<DesktopSession, DesktopError> {
    if session.state == DesktopSessionState::Disconnected {
        return Err(DesktopError::InvalidSession(
            "desktop session is already disconnected".to_string(),
        ));
    }

    let mut next = session.clone();
    next.state = DesktopSessionState::Disconnected;
    next.updated_at = now;
    Ok(next)
}

fn validate_client_version(value: String) -> Result<String, DesktopError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopError::InvalidSession(
            "desktop client version cannot be empty".to_string(),
        ));
    }
    Ok(value)
}
