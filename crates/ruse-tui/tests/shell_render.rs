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

    // Welcome / chrome — brand mark + facts (no dedicated transcript title).
    assert!(rendered.contains("RUSE") || rendered.contains("Workspace"));
    assert!(rendered.contains("fake / fake-scripted") || rendered.contains("fake-scripted"));
    assert!(rendered.contains("workspace") || rendered.contains("/workspace"));
    assert!(rendered.contains("type a message") || rendered.contains("commands"));
    assert!(
        !rendered.contains(" ready"),
        "welcome model line must not show ready: {rendered}"
    );
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
fn shell_render_shows_status_popup() {
    use ruse_tui::{ComposerAction, TuiEffect};

    let mut app = TuiApp::new(StatusBarState::new(
        "openai",
        "gpt-5",
        "/workspace",
        "tui",
        "connected",
        "default-deny",
    ));
    app.reduce(TuiAction::Resize {
        width: 80,
        height: 24,
    });

    app.reduce(ComposerAction::InsertText("/status".to_string()).into());
    let effect = app.reduce(ComposerAction::Submit.into());
    assert!(matches!(effect, TuiEffect::RecoveryStatus { .. }));
    assert!(app.status_panel.is_open());

    let rendered = render_app_to_string(&app, 80, 24).expect("render status popup");
    assert!(rendered.contains("Session status"));
    assert!(rendered.contains("Connection:"));
    assert!(rendered.contains("Enter/Esc close"));
    // Rounded border corners.
    assert!(rendered.contains('╭') && rendered.contains('╯'));
}

#[test]
fn shell_render_replaces_help_panel_with_command_menu() {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::OpenCommands);

    let rendered = render_app_to_string(&app, 72, 18).expect("render command menu");

    assert!(rendered.contains("/model"));
    assert!(rendered.contains("Ctrl+G"));
    assert!(rendered.contains("Ctrl+S"));
    assert!(!rendered.contains("agent Help"));
}

/// When the footer is tight because the workspace path is long, drop
/// provider/model before clipping the workspace. Fixed percentage columns used
/// to keep `provider / model` and compress the path on the right.
#[test]
fn shell_render_status_footer_drops_provider_before_workspace() {
    let mut app = TuiApp::new(StatusBarState::new(
        "openai",
        "gpt-5.5",
        "/Users/dp/very/long/path/to/my/workspace/project",
        "tui",
        "idle",
        "default-deny",
    ));
    app.reduce(TuiAction::Resize {
        width: 88,
        height: 20,
    });

    let rendered = render_app_to_string(&app, 88, 20).expect("render app");
    let footer = rendered.lines().last().expect("footer line").trim_end();

    // Workspace path must remain fully visible (short form keeps the tail).
    assert!(
        footer.contains("workspace/project") || footer.contains("/workspace/project"),
        "workspace was compressed away: {footer:?}"
    );
    // Provider/model is the first optional left segment and must yield first.
    assert!(
        !footer.contains("openai") && !footer.contains("gpt-5.5"),
        "provider/model should drop before workspace is clipped: {footer:?}"
    );
    // Floor left chrome still present.
    assert!(
        footer.contains("xhigh") && footer.contains("normal"),
        "effort + mode must remain: {footer:?}"
    );
}
