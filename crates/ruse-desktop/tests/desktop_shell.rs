use ruse_desktop::{
    DesktopError, DesktopRoute, DesktopSessionState, disconnect_desktop_session,
    start_desktop_session, switch_desktop_route,
};
use ruse_types::{Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000001"
        .parse()
        .expect("valid workspace id")
}

#[test]
fn desktop_shell_serializes_session_shape() {
    let session = start_desktop_session(workspace_id(), "0.1.0", ts(1)).unwrap();

    let value = serde_json::to_value(session).expect("serialize desktop session");

    assert_eq!(
        value["workspace_id"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(value["client_version"], "0.1.0");
    assert_eq!(value["route"], "dashboard");
    assert_eq!(value["state"], "connected");
    assert_eq!(value["connected_at"], "2026-06-25T00:00:01Z");
    assert_eq!(value["updated_at"], "2026-06-25T00:00:01Z");
}

#[test]
fn desktop_shell_starts_on_dashboard_route() {
    let session = start_desktop_session(workspace_id(), "0.1.0", ts(1)).unwrap();

    assert_eq!(session.route, DesktopRoute::Dashboard);
    assert_eq!(session.state, DesktopSessionState::Connected);
    assert_eq!(session.connected_at, ts(1));
    assert_eq!(session.updated_at, ts(1));
}

#[test]
fn desktop_shell_switches_routes() {
    let session = start_desktop_session(workspace_id(), "0.1.0", ts(1)).unwrap();

    let switched = switch_desktop_route(&session, DesktopRoute::Trace, ts(2)).unwrap();

    assert_eq!(switched.session_id, session.session_id);
    assert_eq!(switched.workspace_id, session.workspace_id);
    assert_eq!(switched.route, DesktopRoute::Trace);
    assert_eq!(switched.state, DesktopSessionState::Connected);
    assert_eq!(switched.connected_at, ts(1));
    assert_eq!(switched.updated_at, ts(2));
}

#[test]
fn desktop_shell_rejects_empty_client_version() {
    let err = start_desktop_session(workspace_id(), " ", ts(1)).unwrap_err();

    assert!(matches!(err, DesktopError::InvalidSession(_)));
}

#[test]
fn desktop_shell_rejects_route_switch_after_disconnect() {
    let session = start_desktop_session(workspace_id(), "0.1.0", ts(1)).unwrap();
    let disconnected = disconnect_desktop_session(&session, ts(2)).unwrap();

    let err = switch_desktop_route(&disconnected, DesktopRoute::Settings, ts(3)).unwrap_err();

    assert_eq!(disconnected.state, DesktopSessionState::Disconnected);
    assert_eq!(disconnected.updated_at, ts(2));
    assert!(matches!(err, DesktopError::InvalidSession(_)));
}
