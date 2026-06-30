use agent_tui::{ComposerAction, SlashCommand, SlashCommandError, TuiApp, TuiEffect, TuiSidePanel};

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
fn sessions_slash_command_opens_session_panel() {
    let mut app = TuiApp::default();

    app.reduce(ComposerAction::InsertText("/sessions".to_string()).into());
    let effect = app.reduce(ComposerAction::Submit.into());

    assert_eq!(effect, TuiEffect::None);
    assert_eq!(app.side_panel, TuiSidePanel::Sessions);
}
