use agent_tui::{
    CommandPanelAction, CommandPanelOutput, CommandPanelState, TuiAction, TuiApp,
    render_app_to_string,
};
use agent_ui_bridge::{UiCommandTestSnapshot, UiJsonFrame};

fn snapshot() -> UiCommandTestSnapshot {
    UiCommandTestSnapshot {
        workspace_id: agent_types::WorkspaceId::new(),
        conversation_id: agent_types::ConversationId::new(),
        command: "cargo test -p agent-core command_test_loop".to_string(),
        cwd: ".".to_string(),
        classify_as_test: true,
        status: "failed".to_string(),
        classification: "test_failure".to_string(),
        exit_code: Some(101),
        stdout: "running tests".to_string(),
        stderr: "test failed".to_string(),
        stdout_byte_count: 13,
        stderr_byte_count: 11,
        timed_out: false,
        cancellable: true,
        cancellation_reason: "command can be cancelled before completion".to_string(),
        observation: "command observation: status=101".to_string(),
        truncated: false,
        redacted: false,
        diagnostic_count: 0,
    }
}

#[test]
fn command_panel_builds_from_command_test_snapshot() {
    let panel = CommandPanelState::from_snapshot(&snapshot(), 200);

    assert_eq!(panel.command, "cargo test -p agent-core command_test_loop");
    assert_eq!(panel.status, "failed");
    assert_eq!(panel.classification, "test_failure");
    assert_eq!(panel.exit_code, Some(101));
    assert_eq!(panel.cwd, ".");
    assert!(panel.classify_as_test);
    assert!(panel.stdout.contains("running tests"));
    assert!(panel.stderr.contains("test failed"));
    assert_eq!(panel.stdout_byte_count, 13);
    assert_eq!(panel.stderr_byte_count, 11);
    assert_eq!(
        panel.cancellation_reason,
        "command can be cancelled before completion"
    );
    assert_eq!(panel.active_output, CommandPanelOutput::Stdout);
    assert!(panel.cancellable);
    assert!(!panel.truncated);
}

#[test]
fn command_panel_truncates_large_output() {
    let mut snapshot = snapshot();
    snapshot.stdout = "x".repeat(120);

    let panel = CommandPanelState::from_snapshot(&snapshot, 24);

    assert_eq!(panel.stdout.len(), 24);
    assert!(panel.truncated);
}

#[test]
fn command_panel_switches_between_stdout_and_stderr_views() {
    let mut panel = CommandPanelState::from_snapshot(&snapshot(), 200);

    assert!(panel.render_text().contains("stdout:"));
    assert!(!panel.render_text().contains("stderr:"));

    panel.reduce(CommandPanelAction::ShowStderr);
    assert_eq!(panel.active_output, CommandPanelOutput::Stderr);
    assert!(panel.render_text().contains("stderr:"));
    assert!(!panel.render_text().contains("stdout:"));

    panel.reduce(CommandPanelAction::ToggleOutput);
    assert_eq!(panel.active_output, CommandPanelOutput::Stdout);
}

#[test]
fn command_panel_renders_command_snapshot_in_side_panel() {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::BridgeFrame(Box::new(
        UiJsonFrame::CommandTestSnapshot(snapshot()),
    )));

    let rendered = render_app_to_string(&app, 100, 20).expect("render app");

    assert!(rendered.contains("Command"));
    assert!(rendered.contains("cargo test"));
    assert!(rendered.contains("cwd: ."));
    assert!(rendered.contains("test_failure"));
    assert!(rendered.contains("exit: 101"));
    assert!(rendered.contains("cancellable: yes"));
    assert!(rendered.contains("stdout bytes: 13"));
    assert!(rendered.contains("running tests"));
}
