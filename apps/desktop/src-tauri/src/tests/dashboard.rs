use ruse_desktop::{
    DesktopConfigSnapshot, DesktopDashboardDiagnostic, DesktopDashboardDiagnosticSeverity,
    DesktopSessionReadinessState,
};
use ruse_ui_bridge::{UiConfigSnapshot, UiModelProviderCapabilities, UiModelProviderSummary};

use crate::{
    bridge::{DesktopRuntimeSession, DesktopStartSessionRequest},
    dashboard::build_dashboard_snapshot_for_session,
};

fn config_snapshot() -> DesktopConfigSnapshot {
    let capabilities =
        UiModelProviderCapabilities::new(true, true, false, false, Some(8_192), Some(1_024), None);
    let provider = UiModelProviderSummary::new("fake", "fake", "fake-scripted", true, capabilities)
        .expect("valid provider");
    let snapshot = UiConfigSnapshot::new("fake", "fake-scripted", vec![provider], Vec::new())
        .expect("valid config");
    DesktopConfigSnapshot::new(snapshot).expect("valid desktop config")
}

#[test]
fn dashboard_snapshot_reports_not_started_without_creating_session_identity() {
    let session = DesktopRuntimeSession::new_for_tests(8);

    let snapshot = build_dashboard_snapshot_for_session(&session, &config_snapshot(), Vec::new())
        .expect("dashboard builds");

    assert_eq!(
        snapshot.session.state,
        DesktopSessionReadinessState::NotStarted
    );
    assert_eq!(snapshot.session.session_id, None);
    assert_eq!(snapshot.session.conversation_id, None);
    assert_eq!(
        snapshot.workspace_id.to_string(),
        "00000000-0000-0000-0000-000000000000"
    );
    assert_eq!(snapshot.task_counts.total, 0);
}

#[test]
fn dashboard_snapshot_uses_connected_session_identity() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    let started = session
        .start(DesktopStartSessionRequest {
            workspace_path: Some("/workspace/agent".to_string()),
            client_version: "0.1.0".to_string(),
        })
        .expect("session starts");

    let snapshot = build_dashboard_snapshot_for_session(&session, &config_snapshot(), Vec::new())
        .expect("dashboard builds");

    assert_eq!(
        snapshot.session.state,
        DesktopSessionReadinessState::Connected
    );
    assert_eq!(
        snapshot.session.session_id.expect("session id").to_string(),
        started.session_id
    );
    assert_eq!(
        snapshot
            .session
            .conversation_id
            .expect("conversation id")
            .to_string(),
        started.conversation_id
    );
    assert_eq!(snapshot.workspace_id.to_string(), started.workspace_id);
    assert_eq!(snapshot.session.pending_frame_count, 1);
}

#[test]
fn dashboard_snapshot_reports_disconnected_session() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session
        .start(DesktopStartSessionRequest {
            workspace_path: None,
            client_version: "0.1.0".to_string(),
        })
        .expect("session starts");
    session.stop().expect("session stops");

    let snapshot = build_dashboard_snapshot_for_session(&session, &config_snapshot(), Vec::new())
        .expect("dashboard builds");

    assert_eq!(
        snapshot.session.state,
        DesktopSessionReadinessState::Disconnected
    );
    assert_eq!(snapshot.session.pending_frame_count, 0);
    assert!(snapshot.session.session_id.is_some());
}

#[test]
fn dashboard_snapshot_redacts_diagnostics() {
    let session = DesktopRuntimeSession::new_for_tests(8);
    let diagnostic = DesktopDashboardDiagnostic::new(
        DesktopDashboardDiagnosticSeverity::Warning,
        "dashboard.secret",
        "token sk-secret password",
    )
    .expect("valid diagnostic");

    let snapshot =
        build_dashboard_snapshot_for_session(&session, &config_snapshot(), vec![diagnostic])
            .expect("dashboard builds");
    let serialized = serde_json::to_string(&snapshot).expect("serialize dashboard");

    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("dashboard.secret"));
}
