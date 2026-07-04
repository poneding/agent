use agent_ui_bridge::{UiJsonFrame, UiTerminalSessionDiagnostic, UiTerminalSessionSummary};

use crate::TuiEffect;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionPanelAction {
    FocusNext,
    FocusPrevious,
    SwitchFocused,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SessionPanelState {
    pub sessions: Vec<UiTerminalSessionSummary>,
    pub diagnostics: Vec<UiTerminalSessionDiagnostic>,
    pub focused_index: Option<usize>,
}

impl SessionPanelState {
    pub fn reduce(&mut self, action: SessionPanelAction) -> TuiEffect {
        match action {
            SessionPanelAction::FocusNext => {
                self.focus_next();
                TuiEffect::None
            }
            SessionPanelAction::FocusPrevious => {
                self.focus_previous();
                TuiEffect::None
            }
            SessionPanelAction::SwitchFocused => self.switch_focused(),
        }
    }

    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        let UiJsonFrame::TerminalSessionSnapshot(snapshot) = frame else {
            return;
        };

        self.sessions = snapshot.sessions.clone();
        self.diagnostics = snapshot.diagnostics.clone();
        self.clamp_focus();
    }

    #[must_use]
    pub fn focused_session(&self) -> Option<&UiTerminalSessionSummary> {
        self.focused_index
            .and_then(|index| self.sessions.get(index))
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        if self.sessions.is_empty() {
            return "No terminal sessions".to_string();
        }

        let mut lines = self
            .sessions
            .iter()
            .enumerate()
            .map(|(index, session)| {
                let marker = if Some(index) == self.focused_index {
                    ">"
                } else {
                    " "
                };
                format!(
                    "{marker} {} {} {} workspace={} provider={}/{}",
                    session.status,
                    session.title,
                    session.conversation_id,
                    session.workspace_root_display,
                    session.provider_id,
                    session.model_id
                )
            })
            .collect::<Vec<_>>();

        lines.extend(self.diagnostics.iter().map(|diagnostic| {
            format!(
                "diagnostic {}:{} {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            )
        }));
        lines.join("\n")
    }

    fn focus_next(&mut self) {
        if self.sessions.is_empty() {
            self.focused_index = None;
            return;
        }
        self.focused_index = Some(match self.focused_index {
            Some(index) => (index + 1) % self.sessions.len(),
            None => 0,
        });
    }

    fn focus_previous(&mut self) {
        if self.sessions.is_empty() {
            self.focused_index = None;
            return;
        }
        self.focused_index = Some(match self.focused_index {
            Some(0) | None => self.sessions.len() - 1,
            Some(index) => index - 1,
        });
    }

    fn switch_focused(&self) -> TuiEffect {
        match self.focused_session() {
            Some(session) => TuiEffect::SwitchSession {
                workspace_id: session.workspace_id,
                conversation_id: session.conversation_id,
            },
            None => TuiEffect::LocalDiagnostic("no terminal session is focused".to_string()),
        }
    }

    fn clamp_focus(&mut self) {
        if self.sessions.is_empty() {
            self.focused_index = None;
            return;
        }
        self.focused_index = Some(self.focused_index.unwrap_or(0).min(self.sessions.len() - 1));
    }
}
