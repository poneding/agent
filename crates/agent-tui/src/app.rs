use agent_ui_bridge::UiJsonFrame;

use crate::{
    ApprovalPanelState, CommandPanelState, ComposerAction, ComposerState, DiffPanelState,
    SessionPanelAction, SessionPanelState, SlashCommand, StatusPanelState, TaskPanelState,
    TracePanelState, TranscriptState, TuiEffect,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TuiScreen {
    Chat,
    Help,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TuiSidePanel {
    #[default]
    Auto,
    Trace,
    Tasks,
    Sessions,
    Status,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusBarState {
    pub provider: String,
    pub model: String,
    pub workspace: String,
    pub mode: String,
    pub connection: String,
    pub permission_state: String,
}

impl StatusBarState {
    #[must_use]
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        workspace: impl Into<String>,
        mode: impl Into<String>,
        connection: impl Into<String>,
        permission_state: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            workspace: workspace.into(),
            mode: mode.into(),
            connection: connection.into(),
            permission_state: permission_state.into(),
        }
    }
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self::new(
            "fake",
            "fake-scripted",
            ".",
            "tui",
            "disconnected",
            "default-deny",
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TuiAction {
    Tick,
    QuitRequested,
    Resize { width: u16, height: u16 },
    OpenHelp,
    OpenChat,
    Composer(ComposerAction),
    SessionPanel(SessionPanelAction),
    BridgeFrame(Box<UiJsonFrame>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiApp {
    pub screen: TuiScreen,
    pub status: StatusBarState,
    pub terminal_size: TerminalSize,
    pub composer: ComposerState,
    pub transcript: TranscriptState,
    pub approval_panel: ApprovalPanelState,
    pub diff_panel: Option<DiffPanelState>,
    pub command_panel: Option<CommandPanelState>,
    pub trace_panel: TracePanelState,
    pub task_panel: TaskPanelState,
    pub session_panel: SessionPanelState,
    pub status_panel: StatusPanelState,
    pub side_panel: TuiSidePanel,
    pub should_quit: bool,
}

impl TuiApp {
    #[must_use]
    pub fn new(status: StatusBarState) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }

    pub fn reduce(&mut self, action: TuiAction) -> TuiEffect {
        match action {
            TuiAction::Tick => TuiEffect::None,
            TuiAction::QuitRequested => {
                self.should_quit = true;
                TuiEffect::None
            }
            TuiAction::Resize { width, height } => {
                self.terminal_size = TerminalSize { width, height };
                TuiEffect::None
            }
            TuiAction::OpenHelp => {
                self.screen = TuiScreen::Help;
                TuiEffect::None
            }
            TuiAction::OpenChat => {
                self.screen = TuiScreen::Chat;
                TuiEffect::None
            }
            TuiAction::Composer(action) => {
                let effect = self.composer.reduce(action);
                self.apply_effect(effect)
            }
            TuiAction::SessionPanel(action) => self.session_panel.reduce(action),
            TuiAction::BridgeFrame(frame) => {
                self.approval_panel.reduce_frame(&frame);
                self.trace_panel.reduce_frame(&frame);
                self.task_panel.reduce_frame(&frame);
                self.session_panel.reduce_frame(&frame);
                self.status_panel.reduce_frame(&frame);
                if let UiJsonFrame::CommandTestSnapshot(snapshot) = frame.as_ref() {
                    self.command_panel = Some(CommandPanelState::from_snapshot(snapshot, 800));
                }
                self.diff_panel = self
                    .approval_panel
                    .focused_request()
                    .and_then(|request| DiffPanelState::maybe_from_approval(request, 800));
                self.transcript.reduce_frame(*frame);
                TuiEffect::None
            }
        }
    }

    fn apply_effect(&mut self, effect: TuiEffect) -> TuiEffect {
        let TuiEffect::RunSlashCommand(command) = effect else {
            return effect;
        };

        self.apply_slash_command(command)
    }

    fn apply_slash_command(&mut self, command: SlashCommand) -> TuiEffect {
        match command {
            SlashCommand::Help => {
                self.screen = TuiScreen::Help;
                TuiEffect::None
            }
            SlashCommand::Status => {
                self.screen = TuiScreen::Chat;
                self.side_panel = TuiSidePanel::Status;
                self.status_panel = StatusPanelState::from_status_bar(
                    &self.status,
                    "status",
                    self.session_panel.sessions.len(),
                    self.session_panel.diagnostics.len(),
                );
                TuiEffect::None
            }
            SlashCommand::Trace => {
                self.screen = TuiScreen::Chat;
                self.side_panel = TuiSidePanel::Trace;
                TuiEffect::None
            }
            SlashCommand::Tasks => {
                self.screen = TuiScreen::Chat;
                self.side_panel = TuiSidePanel::Tasks;
                TuiEffect::None
            }
            SlashCommand::Sessions => {
                self.screen = TuiScreen::Chat;
                self.side_panel = TuiSidePanel::Sessions;
                TuiEffect::None
            }
            SlashCommand::Settings => TuiEffect::LocalDiagnostic(
                "settings panel is not available in this milestone".to_string(),
            ),
            SlashCommand::Clear => {
                self.transcript = TranscriptState::default();
                TuiEffect::None
            }
            SlashCommand::Quit => {
                self.should_quit = true;
                TuiEffect::None
            }
        }
    }
}

impl From<ComposerAction> for TuiAction {
    fn from(value: ComposerAction) -> Self {
        Self::Composer(value)
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            screen: TuiScreen::Chat,
            status: StatusBarState::default(),
            terminal_size: TerminalSize::default(),
            composer: ComposerState::default(),
            transcript: TranscriptState::default(),
            approval_panel: ApprovalPanelState::default(),
            diff_panel: None,
            command_panel: None,
            trace_panel: TracePanelState::default(),
            task_panel: TaskPanelState::default(),
            session_panel: SessionPanelState::default(),
            status_panel: StatusPanelState::default(),
            side_panel: TuiSidePanel::default(),
            should_quit: false,
        }
    }
}
