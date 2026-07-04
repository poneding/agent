use agent_cli::{SlashCommand, handle_slash_command};

#[test]
fn slash_commands_parse_known_commands() {
    assert_eq!(
        SlashCommand::parse("/help").expect("help"),
        SlashCommand::Help
    );
    assert_eq!(
        SlashCommand::parse("/status").expect("status"),
        SlashCommand::Status
    );
    assert_eq!(
        SlashCommand::parse("/sessions").expect("sessions"),
        SlashCommand::Sessions
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
}

#[test]
fn slash_commands_render_catalog_help_status_and_exit() {
    let help = handle_slash_command("/help");
    let status = handle_slash_command("/status");
    let sessions = handle_slash_command("/sessions");
    let exit = handle_slash_command("/exit");

    assert!(help.stdout.contains("Commands:"));
    assert!(help.stdout.contains("/sessions"));
    assert!(help.stdout.contains("alias: /exit"));
    assert!(!help.exit);
    assert!(status.stdout.contains("Status: ready"));
    assert!(status.stdout.contains("Provider: fake / fake-scripted"));
    assert!(!status.exit);
    assert!(sessions.stdout.contains("repo session list"));
    assert!(!sessions.exit);
    assert_eq!(exit.stdout, "Bye\n");
    assert!(exit.exit);
}

#[test]
fn slash_commands_report_unknown_command() {
    let output = handle_slash_command("/nope-secret-token");

    assert_eq!(output.stdout, "Unknown command: [redacted]\n");
    assert!(!output.exit);
}
