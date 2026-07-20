use ruse_ui_bridge::{
    UiJsonFrame, UiSlashCommandEffect, UiSlashCommandSurface, UiTerminalStatusSnapshot,
    default_slash_command_catalog, encode_json_frame, resolve_slash_command,
};

#[test]
fn slash_catalog_is_deterministic_and_covers_aliases() {
    let catalog = default_slash_command_catalog();
    let names = catalog
        .commands
        .iter()
        .map(|command| command.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "help",
            "memory",
            "model",
            "permissions",
            "status",
            "resume",
            "sessions",
            "diff",
            "plan",
            "tasks",
            "trace",
            "mcp",
            "tools",
            "clear",
            "compact",
            "copy",
            "quit",
            "retry",
            "settings",
            "theme",
            "thinking"
        ]
    );
    let quit = catalog
        .commands
        .iter()
        .find(|command| command.name == "quit")
        .expect("quit command");
    assert_eq!(quit.aliases, vec!["exit"]);
    assert_eq!(quit.effect, UiSlashCommandEffect::Quit);
    assert!(
        quit.available_in
            .contains(&UiSlashCommandSurface::PlainRepl)
    );
    assert!(quit.available_in.contains(&UiSlashCommandSurface::Tui));
}

#[test]
fn slash_catalog_resolves_canonical_names_and_aliases() {
    let quit = resolve_slash_command(" /exit ").expect("alias resolves");
    let sessions = resolve_slash_command("/sessions").expect("sessions resolves");
    let resume = resolve_slash_command("/resume").expect("resume resolves");
    let model = resolve_slash_command("/model").expect("model resolves");
    let permissions = resolve_slash_command("/permission").expect("permission alias resolves");
    let retry = resolve_slash_command("/retry").expect("retry resolves");
    let copy = resolve_slash_command("/copy").expect("copy resolves");
    let theme = resolve_slash_command("/theme").expect("theme resolves");
    let thinking = resolve_slash_command("/thinking hide-all").expect("thinking resolves");

    assert_eq!(quit.name, "quit");
    assert_eq!(quit.effect, UiSlashCommandEffect::Quit);
    assert_eq!(sessions.effect, UiSlashCommandEffect::OpenSessions);
    assert_eq!(resume.effect, UiSlashCommandEffect::ResumeSession);
    assert_eq!(model.effect, UiSlashCommandEffect::ShowModel);
    assert_eq!(permissions.effect, UiSlashCommandEffect::ShowPermissions);
    assert_eq!(retry.effect, UiSlashCommandEffect::RetryLastTurn);
    assert_eq!(copy.effect, UiSlashCommandEffect::CopyLatestOutput);
    assert_eq!(theme.effect, UiSlashCommandEffect::OpenTheme);
    assert_eq!(thinking.effect, UiSlashCommandEffect::ConfigureThinking);
    assert!(resolve_slash_command("status").is_err());
    assert!(resolve_slash_command("/unknown-secret-token").is_err());
}

#[test]
fn terminal_status_snapshot_bounds_and_redacts_text() {
    let snapshot = UiTerminalStatusSnapshot::new(
        "provider-secret",
        "model-token",
        "/tmp/api_key/project",
        "plain_repl",
        "connected",
        "sessions",
        2,
        1,
    );
    let rendered = serde_json::to_string(&snapshot).expect("serialize status");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("provider-secret"));
    assert!(!rendered.contains("model-token"));
    assert!(!rendered.contains("api_key"));
    assert!(snapshot.workspace.chars().count() <= 120);
}

#[test]
fn slash_catalog_and_status_json_frames_are_single_line() {
    let catalog = default_slash_command_catalog();
    let status = UiTerminalStatusSnapshot::new(
        "fake",
        "fake-scripted",
        ".",
        "tui",
        "connected",
        "status",
        0,
        0,
    );

    let catalog_line =
        encode_json_frame(&UiJsonFrame::SlashCommandCatalogSnapshot(catalog)).expect("catalog");
    let status_line =
        encode_json_frame(&UiJsonFrame::TerminalStatusSnapshot(status)).expect("status");

    assert_eq!(catalog_line.matches('\n').count(), 1);
    assert_eq!(status_line.matches('\n').count(), 1);
    assert!(matches!(
        serde_json::from_str::<UiJsonFrame>(catalog_line.trim_end()).expect("catalog frame"),
        UiJsonFrame::SlashCommandCatalogSnapshot(_)
    ));
    assert!(matches!(
        serde_json::from_str::<UiJsonFrame>(status_line.trim_end()).expect("status frame"),
        UiJsonFrame::TerminalStatusSnapshot(_)
    ));
}
