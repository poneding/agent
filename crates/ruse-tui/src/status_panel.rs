use ruse_ui_bridge::{UiJsonFrame, UiProviderSupportSummary, UiTerminalStatusSnapshot};

use crate::app::StatusBarState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusPanelState {
    pub snapshot: UiTerminalStatusSnapshot,
    pub provider_support: Vec<UiProviderSupportSummary>,
    /// When true, the status is shown as a centered popup (same shape as the
    /// model/session/theme modals). Opened by `/status`, dismissed with Esc.
    pub open: bool,
}

impl StatusPanelState {
    #[must_use]
    pub fn from_status_bar_with_theme(
        status: &crate::StatusBarState,
        focused_panel: &str,
        session_count: usize,
        diagnostic_count: usize,
        _theme: &str,
    ) -> Self {
        Self::from_status_bar(status, focused_panel, session_count, diagnostic_count)
    }

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
            provider_support: Vec::new(),
            open: false,
        }
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::TerminalStatusSnapshot(snapshot) => {
                self.snapshot = snapshot.clone();
            }
            UiJsonFrame::ConfigSnapshot(snapshot) => {
                self.provider_support = snapshot.provider_support.clone();
            }
            _ => {}
        }
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
