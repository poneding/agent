use ruse_tui::{
    FixtureTerminalBackend, StatusBarState, TerminalModeGuard, TerminalSize, TuiAction, TuiApp,
    TuiScreen,
};

#[test]
fn shell_state_starts_on_chat_screen_with_stable_status() {
    let app = TuiApp::new(StatusBarState::new(
        "fake",
        "fake-scripted",
        "/workspace",
        "tui",
        "disconnected",
        "default-deny",
    ));

    assert_eq!(app.screen, TuiScreen::Chat);
    assert_eq!(app.status.provider, "fake");
    assert_eq!(app.status.model, "fake-scripted");
    assert_eq!(app.status.workspace, "/workspace");
    assert_eq!(app.status.mode, "tui");
    assert_eq!(app.status.connection, "disconnected");
    assert_eq!(app.status.permission_state, "default-deny");
    assert!(!app.should_quit);
}

#[test]
fn shell_state_reducer_handles_resize_help_and_quit() {
    let mut app = TuiApp::default();

    app.reduce(TuiAction::Resize {
        width: 100,
        height: 30,
    });
    assert_eq!(
        app.terminal_size,
        TerminalSize {
            width: 100,
            height: 30
        }
    );

    app.reduce(TuiAction::OpenHelp);
    assert_eq!(app.screen, TuiScreen::Help);

    app.reduce(TuiAction::OpenChat);
    assert_eq!(app.screen, TuiScreen::Chat);

    app.reduce(TuiAction::QuitRequested);
    assert!(app.should_quit);
}

#[test]
fn terminal_mode_guard_restores_terminal_on_drop() {
    let backend = FixtureTerminalBackend::default();
    let mut guard = TerminalModeGuard::enter(backend).expect("enter terminal mode");

    assert_eq!(
        guard.backend().calls(),
        &[
            "enable_raw_mode",
            "enter_alternate_screen",
            "enable_mouse_capture",
        ]
    );

    guard.restore().expect("restore once");
    assert_eq!(
        guard.backend().calls(),
        &[
            "enable_raw_mode",
            "enter_alternate_screen",
            "enable_mouse_capture",
            "disable_mouse_capture",
            "leave_alternate_screen",
            "disable_raw_mode",
        ]
    );

    drop(guard);
}

#[test]
fn terminal_mode_guard_drop_restores_when_not_restored_explicitly() {
    let backend = FixtureTerminalBackend::default();
    let mut guard = TerminalModeGuard::enter(backend).expect("enter terminal mode");
    guard.backend_mut().clear_calls();

    let backend = guard.into_inner_restoring().expect("restore into backend");

    assert_eq!(
        backend.calls(),
        &[
            "disable_mouse_capture",
            "leave_alternate_screen",
            "disable_raw_mode",
        ]
    );
}
