use ruse_tui::{
    ComposerAction, SlashCommand, SlashCommandError, SlashSuggestionState, TranscriptItem,
    TranscriptItemKind, TuiApp, TuiEffect, TuiScreen,
};

#[test]
fn slash_parser_accepts_known_commands() {
    assert_eq!(
        SlashCommand::parse("/help").expect("help"),
        SlashCommand::Help
    );
    assert_eq!(
        SlashCommand::parse("/status").expect("status"),
        SlashCommand::Status
    );
    assert_eq!(
        SlashCommand::parse("/trace").expect("trace"),
        SlashCommand::Trace
    );
    assert_eq!(
        SlashCommand::parse("/tasks").expect("tasks"),
        SlashCommand::Tasks
    );
    assert_eq!(
        SlashCommand::parse("/sessions").expect("sessions"),
        SlashCommand::Sessions
    );
    assert_eq!(SlashCommand::parse("/new").expect("new"), SlashCommand::New);
    assert_eq!(
        SlashCommand::parse("/settings").expect("settings"),
        SlashCommand::Settings
    );
    assert_eq!(
        SlashCommand::parse("/clear").expect("clear"),
        SlashCommand::Clear
    );
    assert_eq!(
        SlashCommand::parse("/quit").expect("quit"),
        SlashCommand::Quit
    );
    assert_eq!(
        SlashCommand::parse("/exit").expect("exit"),
        SlashCommand::Quit
    );
    assert_eq!(
        SlashCommand::parse("/bye").expect("bye"),
        SlashCommand::Quit
    );
}

#[test]
fn slash_parser_trims_input_and_rejects_non_commands() {
    assert_eq!(
        SlashCommand::parse("  /help  ").expect("trimmed"),
        SlashCommand::Help
    );

    let error = SlashCommand::parse("help").expect_err("missing slash rejected");

    assert_eq!(error, SlashCommandError::MissingSlash);
}

#[test]
fn slash_parser_reports_unknown_command_without_runtime_effect() {
    let error = SlashCommand::parse("/deploy").expect_err("unknown command rejected");

    assert_eq!(
        error,
        SlashCommandError::UnknownCommand {
            command: "/deploy".to_string()
        }
    );
}

#[test]
fn slash_commands_map_to_tui_effects() {
    assert_eq!(
        TuiEffect::from_slash_command(SlashCommand::Help),
        TuiEffect::RunSlashCommand(SlashCommand::Help)
    );
    assert_eq!(
        TuiEffect::from_slash_command(SlashCommand::Clear),
        TuiEffect::RunSlashCommand(SlashCommand::Clear)
    );
    assert_eq!(
        TuiEffect::from_slash_command(SlashCommand::Quit),
        TuiEffect::RunSlashCommand(SlashCommand::Quit)
    );
}

#[test]
fn help_slash_command_opens_command_menu_instead_of_a_panel() {
    let mut app = TuiApp::default();

    app.reduce(ComposerAction::InsertText("/help".to_string()).into());
    let effect = app.reduce(ComposerAction::Submit.into());

    assert_eq!(effect, TuiEffect::None);
    assert_eq!(app.screen, TuiScreen::Chat);
    assert!(app.composer.slash_suggestions.visible());
    assert!(
        app.composer
            .slash_suggestions
            .matches
            .iter()
            .all(|item| item.name != "help")
    );
}

#[test]
fn slash_suggestions_include_matching_keyboard_shortcuts() {
    let suggestions = SlashSuggestionState::from_input("/", 0);
    let summary = |name: &str| {
        suggestions
            .matches
            .iter()
            .find(|item| item.name == name)
            .map(|item| item.summary.as_str())
            .expect("command suggestion")
    };

    assert_eq!(summary("model"), "Pick model (Ctrl+G)");
    assert_eq!(summary("sessions"), "Sessions (Ctrl+S)");
    assert_eq!(summary("theme"), "Theme (Ctrl+T)");
    assert_eq!(summary("clear"), "Clear transcript (Ctrl+L)");
    assert_eq!(summary("quit"), "Quit (Ctrl+C when idle)");
}

#[test]
fn sessions_slash_command_opens_session_panel() {
    let mut app = TuiApp::default();

    app.reduce(ComposerAction::InsertText("/sessions".to_string()).into());
    let effect = app.reduce(ComposerAction::Submit.into());

    assert_eq!(effect, TuiEffect::None);
    assert!(app.session_panel.is_open());
}

#[test]
fn new_slash_command_starts_a_fresh_session() {
    let mut app = TuiApp::default();
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::User,
        "user",
        "previous conversation",
        false,
    ));

    app.reduce(ComposerAction::InsertText("/new".to_string()).into());
    let effect = app.reduce(ComposerAction::Submit.into());

    assert_eq!(effect, TuiEffect::NewSession);
    assert!(app.transcript.items.is_empty());
    assert!(!app.session_panel.is_open());
}

#[test]
fn status_slash_command_refreshes_status_state() {
    let mut app = TuiApp::default();

    app.reduce(ComposerAction::InsertText("/status".to_string()).into());
    let effect = app.reduce(ComposerAction::Submit.into());

    assert!(matches!(
        effect,
        TuiEffect::RecoveryStatus { turn_id: None }
    ));
    assert_eq!(app.status_panel.snapshot.provider, "fake");
    assert_eq!(app.status_panel.snapshot.focused_panel, "status");
    // `/status` now surfaces the snapshot as a centered popup.
    assert!(app.status_panel.is_open());
}
