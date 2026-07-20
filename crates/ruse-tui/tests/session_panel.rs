use ruse_tui::{SessionPanelAction, SessionPanelState, TuiBridgeContext, TuiEffect};
use ruse_types::{ClientKind, ConversationId, Timestamp, WorkspaceId};
use ruse_ui_bridge::{
    UiCommand, UiJsonFrame, UiTerminalSessionSnapshot, UiTerminalSessionSummary,
};

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn summary(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    title: &str,
    status: &str,
) -> UiTerminalSessionSummary {
    UiTerminalSessionSummary {
        workspace_id,
        conversation_id,
        workspace_root_display: format!("workspace-{conversation_id}"),
        title: title.to_string(),
        client_kind: "cli".to_string(),
        client_version: "0.1.0".to_string(),
        provider_id: "fake".to_string(),
        model_id: "fake-scripted".to_string(),
        git_state: "clean".to_string(),
        last_event_at: Timestamp::now_utc(),
        status: status.to_string(),
        pending_approval_count: 0,
        running_command_count: 0,
        task_count: 0,
    }
}

fn snapshot() -> UiJsonFrame {
    UiJsonFrame::TerminalSessionSnapshot(UiTerminalSessionSnapshot {
        sessions: vec![
            summary(
                id("00000000-0000-0000-0000-000000000001"),
                id("00000000-0000-0000-0000-000000000011"),
                "first",
                "running_command",
            ),
            summary(
                id("00000000-0000-0000-0000-000000000002"),
                id("00000000-0000-0000-0000-000000000022"),
                "second",
                "idle",
            ),
        ],
        diagnostics: Vec::new(),
    })
}

#[test]
fn session_panel_reduces_snapshot_and_renders_focus() {
    let mut panel = SessionPanelState::default();

    panel.reduce_frame(&snapshot());

    assert_eq!(panel.sessions.len(), 2);
    assert_eq!(panel.focused_index, Some(0));
    let rendered = panel.render_text();
    assert!(rendered.contains("> running_command first"));
    assert!(rendered.contains("  idle second"));
}

#[test]
fn session_panel_moves_focus_with_wraparound() {
    let mut panel = SessionPanelState::default();
    panel.reduce_frame(&snapshot());

    assert_eq!(panel.reduce(SessionPanelAction::FocusNext), TuiEffect::None);
    assert_eq!(panel.focused_index, Some(1));
    assert_eq!(panel.reduce(SessionPanelAction::FocusNext), TuiEffect::None);
    assert_eq!(panel.focused_index, Some(0));
    assert_eq!(
        panel.reduce(SessionPanelAction::FocusPrevious),
        TuiEffect::None
    );
    assert_eq!(panel.focused_index, Some(1));
}

#[test]
fn session_panel_emits_switch_session_effect() {
    let mut panel = SessionPanelState::default();
    panel.reduce_frame(&snapshot());

    let effect = panel.reduce(SessionPanelAction::SwitchFocused);

    assert_eq!(
        effect,
        TuiEffect::SwitchSession {
            workspace_id: id("00000000-0000-0000-0000-000000000001"),
            conversation_id: id("00000000-0000-0000-0000-000000000011")
        }
    );

    let context = TuiBridgeContext::new(
        WorkspaceId::new(),
        ConversationId::new(),
        ClientKind::Tui,
        "ruse-tui-test",
    )
    .expect("context");
    let envelope = effect
        .to_ui_command(&context)
        .expect("switch validates")
        .expect("switch maps to command");

    assert!(matches!(
        envelope.command,
        UiCommand::SwitchSession {
            workspace_id,
            conversation_id
        } if workspace_id == id("00000000-0000-0000-0000-000000000001")
            && conversation_id == id("00000000-0000-0000-0000-000000000011")
    ));
}

#[test]
fn session_panel_empty_state_is_stable() {
    let mut panel = SessionPanelState::default();

    assert_eq!(panel.render_text(), "No terminal sessions");
    assert_eq!(
        panel.reduce(SessionPanelAction::SwitchFocused),
        TuiEffect::LocalDiagnostic("no terminal session is focused".to_string())
    );
}
