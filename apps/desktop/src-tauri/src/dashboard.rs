use ruse_desktop::{
    DesktopConfigSnapshot, DesktopDashboardDiagnostic, DesktopDashboardInput,
    DesktopDashboardSnapshot, build_dashboard_snapshot,
};

use crate::{
    bridge::DesktopRuntimeSession, commands::DesktopCommandError,
    workspace_snapshot::build_workspace_snapshot_response,
};

pub fn build_dashboard_snapshot_for_session(
    session: &DesktopRuntimeSession,
    config: &DesktopConfigSnapshot,
    diagnostics: Vec<DesktopDashboardDiagnostic>,
) -> Result<DesktopDashboardSnapshot, DesktopCommandError> {
    let (workspace_id, active_conversation_id) = session.workspace_snapshot_identity();
    let workspace = build_workspace_snapshot_response(
        workspace_id,
        active_conversation_id,
        Vec::new(),
        Vec::new(),
    )?;
    build_dashboard_snapshot(DesktopDashboardInput {
        workspace,
        workspace_root_display: None,
        session: session.readiness(),
        config: config.clone(),
        pending_approval_count: 0,
        diagnostics,
    })
    .map_err(|error| DesktopCommandError::new("desktop.dashboard.invalid", error.to_string()))
}
