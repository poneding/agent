use std::cell::RefCell;

use ruse_cli::{
    CliCommand, CliInvocation, OutputMode, PlainReplEvent, PlainReplRuntimeError,
    PlainReplSymbolMode, StdioProfile, parse_cli_command, parse_cli_invocation,
    parse_plain_repl_line, render_approval_prompt, render_cli_invocation_with_stdio,
    render_plain_repl_approval_prompt, render_plain_repl_footer,
    render_plain_repl_footer_with_width, render_plain_repl_footer_with_width_and_symbols,
    render_plain_repl_frame, render_plain_repl_frame_with_symbols,
    render_plain_repl_interactive_frame, render_plain_repl_interactive_frame_with_symbols,
    render_plain_repl_interactive_screen, render_plain_repl_prompt,
    render_plain_repl_prompt_with_width, render_plain_repl_script, render_plain_repl_separator,
    run_plain_repl_io_with_status_mode,
};
use ruse_ui_bridge::{UiApprovalPreviewSummary, UiApprovalRequest};

#[test]
fn plain_repl_command_parsing_is_script_safe() {
    assert_eq!(
        parse_cli_command(["repl", "--plain"]).expect("parse repl --plain"),
        CliCommand::PlainRepl
    );
    assert_eq!(
        parse_cli_command(["repl", "plain"]).expect("parse repl plain"),
        CliCommand::PlainRepl
    );

    let no_args = parse_cli_invocation(std::iter::empty::<&str>()).expect("parse no args");
    let non_tty = render_cli_invocation_with_stdio(
        no_args,
        StdioProfile {
            stdin_tty: false,
            stdout_tty: false,
        },
    );
    assert!(non_tty.exit);
    assert_ne!(non_tty.status_code, 0);
    assert!(non_tty.stdout.contains("usage: agent"));

    let json_prompt = parse_cli_invocation(["--json", "--prompt", "hello"]).expect("json prompt");
    assert_eq!(
        json_prompt.command,
        CliCommand::BatchPrompt {
            prompt: "hello".to_string()
        }
    );
    assert_eq!(json_prompt.output_mode, OutputMode::Json);
}

#[test]
fn plain_repl_line_parser_classifies_prompt_slash_empty_and_exit() {
    assert_eq!(render_plain_repl_separator(12), "────────────");
    assert_eq!(render_plain_repl_separator(0), "─");
    assert_eq!(render_plain_repl_prompt_with_width(12), "────────────\n❯ ");
    assert_eq!(
        render_plain_repl_prompt(),
        format!("{}\n❯ ", "─".repeat(80))
    );
    assert_eq!(
        render_plain_repl_footer_with_width("GPT 5.5 xhigh", 12),
        "────────────\n 🧠 GPT 5.5 xhigh\n"
    );
    assert_eq!(
        render_plain_repl_footer("GPT 5.5 xhigh"),
        format!("{}\n 🧠 GPT 5.5 xhigh\n", "─".repeat(80))
    );
    assert_eq!(
        render_plain_repl_frame("GPT 5.5 xhigh", 12),
        "────────────\n❯ \n────────────\n 🧠 GPT 5.5 xhigh\n"
    );
    assert_eq!(
        render_plain_repl_footer_with_width_and_symbols(
            "GPT 5.5 xhigh",
            12,
            PlainReplSymbolMode::NoEmoji
        ),
        "────────────\n [model] GPT 5.5 xhigh\n"
    );
    assert_eq!(
        render_plain_repl_frame_with_symbols("GPT 5.5 xhigh", 12, PlainReplSymbolMode::NoEmoji),
        "────────────\n❯ \n────────────\n [model] GPT 5.5 xhigh\n"
    );
    assert_eq!(
        render_plain_repl_interactive_frame("GPT 5.5 xhigh", 12),
        "────────────\n❯ \u{1b}[s\n────────────\n 🧠 GPT 5.5 xhigh\n\u{1b}[u"
    );
    assert_eq!(
        render_plain_repl_interactive_frame_with_symbols(
            "GPT 5.5 xhigh",
            12,
            PlainReplSymbolMode::NoEmoji
        ),
        "────────────\n❯ \u{1b}[s\n────────────\n [model] GPT 5.5 xhigh\n\u{1b}[u"
    );
    assert_eq!(
        render_plain_repl_interactive_screen("GPT 5.5 xhigh", 12),
        "\u{1b}[2J\u{1b}[H────────────\n❯ \u{1b}[s\n────────────\n 🧠 GPT 5.5 xhigh\n\u{1b}[u"
    );
    assert_eq!(parse_plain_repl_line("   "), None);
    assert_eq!(
        parse_plain_repl_line("hello agent"),
        Some(PlainReplEvent::PromptSubmitted("hello agent".to_string()))
    );
    assert_eq!(
        parse_plain_repl_line(" /status "),
        Some(PlainReplEvent::SlashCommand("/status".to_string()))
    );
    assert_eq!(parse_plain_repl_line("/exit"), Some(PlainReplEvent::Exit));
    assert_eq!(parse_plain_repl_line("/quit"), Some(PlainReplEvent::Exit));
}

#[test]
fn plain_repl_fixture_submits_prompts_and_exits_on_slash_exit() {
    let seen_prompts = RefCell::new(Vec::new());
    let output = render_plain_repl_script("hello\n/exit\n", |prompt| {
        seen_prompts.borrow_mut().push(prompt.to_string());
        Ok(format!("fake response: {prompt}\n"))
    });

    assert!(output.exit);
    assert_eq!(output.status_code, 0);
    assert_eq!(seen_prompts.into_inner(), vec!["hello"]);
    assert!(!output.stdout.contains("agent plain repl"));
    assert!(output.stdout.contains(&"─".repeat(80)));
    assert!(output.stdout.contains("❯ "));
    assert!(output.stdout.contains("🧠 GPT 5.5 xhigh"));
    assert!(output.stdout.contains("fake response: hello"));
    assert!(output.stdout.contains("Bye"));
}

#[test]
fn plain_repl_interactive_mode_uses_alternate_screen_and_transcript_records() {
    let mut stdout = Vec::new();
    let status_code = run_plain_repl_io_with_status_mode(
        std::io::Cursor::new("hello\n/exit\n"),
        &mut stdout,
        "GPT 5.5 xhigh",
        true,
        |prompt| Ok(format!("fake response: {prompt}\n")),
    );
    let stdout = String::from_utf8(stdout).expect("utf-8 output");

    assert_eq!(status_code, 0);
    assert!(stdout.starts_with("\u{1b}[?1049h"));
    assert!(stdout.contains("\u{1b}[2J\u{1b}[H"));
    assert!(stdout.contains("\u{1b}[48;5;236m\u{1b}[38;5;231m You \u{1b}[0m"));
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("Agent\nfake response: hello"));
    assert!(stdout.ends_with("\u{1b}[?1049l"));
    assert!(!stdout.contains("Bye\n"));
}

#[test]
fn plain_repl_fixture_handles_slash_commands_without_prompt_submission() {
    let seen_prompts = RefCell::new(Vec::<String>::new());
    let output = render_plain_repl_script("/status\n/exit\n", |prompt| {
        seen_prompts.borrow_mut().push(prompt.to_string());
        Ok(format!("unexpected: {prompt}\n"))
    });

    assert_eq!(seen_prompts.into_inner(), Vec::<String>::new());
    assert!(output.stdout.contains("Status: ready"));
    assert!(output.stdout.contains("Provider: fake / fake-scripted"));
    assert!(output.stdout.contains("Bye"));
}

#[test]
fn plain_repl_fixture_ignores_empty_lines_and_exits_cleanly_on_eof() {
    let seen_prompts = RefCell::new(Vec::<String>::new());
    let output = render_plain_repl_script("\n  \n", |prompt| {
        seen_prompts.borrow_mut().push(prompt.to_string());
        Ok(format!("unexpected: {prompt}\n"))
    });

    assert!(output.exit);
    assert_eq!(output.status_code, 0);
    assert_eq!(seen_prompts.into_inner(), Vec::<String>::new());
    assert!(!output.stdout.contains("unexpected"));
}

#[test]
fn plain_repl_fixture_recovers_from_recoverable_runtime_diagnostics() {
    let output = render_plain_repl_script("fail\nok\n/exit\n", |prompt| {
        if prompt == "fail" {
            Err(PlainReplRuntimeError::recoverable("provider disconnected"))
        } else {
            Ok(format!("fake response: {prompt}\n"))
        }
    });

    assert!(output.exit);
    assert_eq!(output.status_code, 0);
    assert!(
        output
            .stdout
            .contains("recoverable error: provider disconnected")
    );
    assert!(output.stdout.contains("fake response: ok"));
}

#[test]
fn plain_repl_fixture_stops_on_terminal_runtime_error() {
    let output = render_plain_repl_script("fail\nok\n", |prompt| {
        if prompt == "fail" {
            Err(PlainReplRuntimeError::terminal("invalid config"))
        } else {
            Ok(format!("unexpected: {prompt}\n"))
        }
    });

    assert!(output.exit);
    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("terminal error: invalid config"));
    assert!(!output.stdout.contains("unexpected"));
}

#[test]
fn plain_repl_invocation_remains_a_text_command() {
    let invocation = parse_cli_invocation(["repl", "--plain"]).expect("plain repl invocation");

    assert_eq!(
        invocation,
        CliInvocation {
            command: CliCommand::PlainRepl,
            output_mode: OutputMode::Text,
            model_selection: Default::default(),
            config_path: None,
            config_override: Default::default(),
        }
    );
}

#[test]
fn plain_repl_reuses_existing_approval_prompt_renderer() {
    let request = UiApprovalRequest {
        request_id: "00000000-0000-0000-0000-000000000001".to_string(),
        status: "pending".to_string(),
        action: "execute".to_string(),
        resource_kind: "command".to_string(),
        resource: "cargo test".to_string(),
        reason: "verify before completion".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "command".to_string(),
            resource: "cargo test".to_string(),
            summary: "Run cargo test".to_string(),
            content_hash: None,
            truncated: false,
            redacted: true,
        },
        max_duration_seconds: Some(300),
        expires_at: None,
    };

    assert_eq!(
        render_plain_repl_approval_prompt(&request),
        render_approval_prompt(&request)
    );
}
