use agent_ui_bridge::{UiJsonFrame, UiTerminalStatusSnapshot};

use crate::app::StatusBarState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusPanelState {
    pub snapshot: UiTerminalStatusSnapshot,
}

impl StatusPanelState {
    #[must_use]
    pub fn from_status_bar(
        status: &StatusBarState,
        focused_panel: &str,
        session_count: usize,
        diagnostic_count: usize,
    ) -> Self {
        Self {
            snapshot: UiTerminalStatusSnapshot::new(
                &status.provider,
                &status.model,
                &status.workspace,
                &status.mode,
                &status.connection,
                focused_panel,
                session_count.try_into().unwrap_or(u32::MAX),
                diagnostic_count.try_into().unwrap_or(u32::MAX),
            ),
        }
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        let UiJsonFrame::TerminalStatusSnapshot(snapshot) = frame else {
            return;
        };
        self.snapshot = snapshot.clone();
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        format!(
            "Status: {}\nProvider: {} / {}\nWorkspace: {}\nMode: {}\nFocused panel: {}\nSessions: {}\nDiagnostics: {}",
            self.snapshot.connection,
            self.snapshot.provider,
            self.snapshot.model,
            self.snapshot.workspace,
            self.snapshot.mode,
            self.snapshot.focused_panel,
            self.snapshot.session_count,
            self.snapshot.diagnostic_count
        )
    }
}

impl Default for StatusPanelState {
    fn default() -> Self {
        Self::from_status_bar(&StatusBarState::default(), "none", 0, 0)
    }
}
