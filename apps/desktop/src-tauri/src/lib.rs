#![forbid(unsafe_code)]
//! Tauri Desktop shell for agent.

pub mod approval_commands;
pub mod bootstrap;
pub mod bridge;
pub mod commands;
pub mod settings_commands;
pub mod state;
pub mod trace_commands;
pub mod workspace_snapshot;

pub use bootstrap::{
    DesktopBootstrap, DesktopBootstrapDiagnostic, DesktopBootstrapError, DesktopRuntimeState,
    build_desktop_bootstrap,
};
pub use bridge::{
    DesktopCommandAck, DesktopRuntimeSession, DesktopSessionSnapshot, DesktopStartSessionRequest,
};
pub use commands::{
    DesktopCommandError, desktop_bootstrap, drain_ui_frames, load_workspace_snapshot,
    resolve_approval, send_ui_command, start_session, stop_session,
};
pub use settings_commands::{load_config_snapshot, validate_settings};
pub use state::DesktopAppState;
pub use trace_commands::load_trace;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopAppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::desktop_bootstrap,
            commands::start_session,
            commands::stop_session,
            commands::send_ui_command,
            commands::drain_ui_frames,
            commands::load_workspace_snapshot,
            commands::resolve_approval,
            trace_commands::load_trace,
            settings_commands::load_config_snapshot,
            settings_commands::validate_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to run agent desktop app");
}

#[cfg(test)]
mod tests {
    mod approval_commands;
    mod bootstrap;
    mod bridge_session;
    mod trace_settings_commands;
    mod workspace_snapshot;
}
