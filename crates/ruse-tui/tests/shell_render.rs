use ruse_tui::{StatusBarState, TuiAction, TuiApp, render_app_to_string};

#[test]
fn shell_render_draws_main_regions_in_normal_layout() {
    let mut app = TuiApp::new(StatusBarState::new(
        "fake",
        "fake-scripted",
        "/workspace",
        "tui",
        "connected",
        "default-deny",
    ));
    app.reduce(TuiAction::Resize {
        width: 80,
        height: 24,
    });

    let rendered = render_app_to_string(&app, 80, 24).expect("render app");

    // Welcome / chrome (no dedicated "Transcript" title in the current layout).
    assert!(rendered.contains("AGENT") || rendered.contains("Workspace"));
    assert!(rendered.contains("fake / fake-scripted") || rendered.contains("fake-scripted"));
    assert!(rendered.contains("/workspace"));
    assert!(rendered.contains("type a message") || rendered.contains("commands"));
}

#[test]
fn shell_render_hides_side_panel_in_narrow_layout() {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::Resize {
        width: 38,
        height: 12,
    });

    let rendered = render_app_to_string(&app, 38, 12).expect("render app");

    assert!(rendered.contains("type a message") || rendered.contains("❯"));
    assert!(!rendered.contains("Panel"));
}

#[test]
fn shell_render_draws_help_screen() {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::OpenHelp);

    let rendered = render_app_to_string(&app, 72, 18).expect("render help");

    assert!(rendered.contains("Help"));
    assert!(rendered.contains("Esc"));
}
