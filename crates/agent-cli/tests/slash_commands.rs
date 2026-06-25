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
        SlashCommand::parse("/exit").expect("exit"),
        SlashCommand::Exit
    );
}

#[test]
fn slash_commands_render_help_status_and_exit() {
    let help = handle_slash_command("/help");
    let status = handle_slash_command("/status");
    let exit = handle_slash_command("/exit");

    assert_eq!(help.stdout, "Commands: /help, /status, /exit\n");
    assert!(!help.exit);
    assert_eq!(status.stdout, "Runtime: ready\n");
    assert!(!status.exit);
    assert_eq!(exit.stdout, "Bye\n");
    assert!(exit.exit);
}

#[test]
fn slash_commands_report_unknown_command() {
    let output = handle_slash_command("/nope");

    assert_eq!(output.stdout, "Unknown command: /nope\n");
    assert!(!output.exit);
}
