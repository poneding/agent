use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use ruse_tui::{
    ApprovalPanelAction, ClipboardCopyReport, ComposerAction, DiagnosticTerminalCommandRuntime,
    DiagnosticTerminalTurnRuntime, PlatformClipboardResult, ReasoningEffort, SessionPanelAction,
    TerminalCancellationToken, TerminalCommandRuntime, TerminalEffectRuntime, TerminalFrameSender,
    TerminalTheme, TerminalTurnOptions, TerminalTurnRuntime, TranscriptItemKind, TuiAction, TuiApp,
    TuiEffect, TurnController, apply_terminal_effect, apply_terminal_effect_with_runtime,
    apply_terminal_effect_with_runtimes, build_terminal_app_state,
    copy_text_to_terminal_clipboard_with_platform_result, drain_terminal_frames,
    handle_cancel_turn_requested, key_event_to_action, key_event_to_action_for_app,
    mouse_event_to_action, osc52_copy_sequence, platform_clipboard_command_candidates_for,
    start_next_queued_prompt, write_osc52_clipboard,
};
use ruse_types::{
    ApprovalRequestId, ConversationId, CorrelationId, EventId, EventSeq, IdempotentRecoverySummary,
    RecoveryContinuation, RecoveryReadiness, RecoveryReplaySummary, RecoverySnapshot, ResumeReport,
    SchemaVersion, TaskId, Timestamp, TurnId,
};
use ruse_ui_bridge::{
    UiApprovalDecision, UiApprovalPreviewSummary, UiApprovalRequest, UiEventEnvelope,
    UiEventPayload, UiJsonFrame, UiResumeReport,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

#[test]
fn terminal_key_mapping_routes_common_composer_controls() {
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        Some(TuiAction::Composer(ComposerAction::InsertChar('a')))
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Some(TuiAction::Composer(ComposerAction::Submit))
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
        Some(TuiAction::Composer(ComposerAction::Backspace))
    );
    // Bare Up/Down scroll the transcript so a mouse wheel (without mouse capture)
    // never cycles prompt history. History is Alt/Ctrl+Up/Down.
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        Some(TuiAction::ScrollTranscriptUp)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Some(TuiAction::ScrollTranscriptDown)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::ALT)),
        Some(TuiAction::Composer(ComposerAction::HistoryPrevious))
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Down, KeyModifiers::ALT)),
        Some(TuiAction::Composer(ComposerAction::HistoryNext))
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL)),
        Some(TuiAction::Composer(ComposerAction::HistoryPrevious))
    );
}

#[test]
fn terminal_key_mapping_routes_global_actions() {
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Some(TuiAction::CancelTurnRequested)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)),
        Some(TuiAction::ClearTranscript)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)),
        Some(TuiAction::TogglePanelExpansion)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL)),
        Some(TuiAction::ToggleToolOutput)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)),
        Some(TuiAction::ToggleReasoning)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
        Some(TuiAction::StartRecoveryContinuation)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)),
        Some(TuiAction::CycleTheme)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
        Some(TuiAction::CyclePermissionMode)
    );
    assert_eq!(
        key_event_to_action_for_app(
            KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
            &TuiApp::default(),
        ),
        Some(TuiAction::CycleReasoningEffort)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)),
        Some(TuiAction::OpenHelp)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
        Some(TuiAction::ScrollTranscriptPageUp)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
        Some(TuiAction::ScrollTranscriptPageDown)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
        Some(TuiAction::ScrollTranscriptTop)
    );
    assert_eq!(
        key_event_to_action(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
        Some(TuiAction::ScrollTranscriptBottom)
    );
}

#[test]
fn terminal_mouse_wheel_scrolls_transcript_and_drag_starts_selection() {
    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }),
        Some(TuiAction::ScrollTranscriptUp)
    );
    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }),
        Some(TuiAction::ScrollTranscriptDown)
    );
    // Left-drag is now owned by in-app selection (native host selection is off
    // under mouse capture). Shift+drag is left alone for host terminals that
    // still synthesize selection.
    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }),
        Some(TuiAction::SelectionDragUpdate {
            column: 0,
            row: 0,
        })
    );
    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::SHIFT,
        }),
        None
    );
}

#[test]
fn terminal_recovery_continuation_requires_explicit_start_action() {
    let mut app = TuiApp::default();
    let conversation_id = ConversationId::new();
    let frame = UiJsonFrame::recovery_resume(UiResumeReport::new(ResumeReport {
        snapshot: RecoverySnapshot {
            conversation_id,
            turn_id: Some(TurnId::new()),
            checkpoint: None,
            replay: RecoveryReplaySummary {
                replay_from: EventSeq::new(1).expect("event seq"),
                replayed_event_count: 1,
            },
            readiness: RecoveryReadiness::Ready,
            pending_tool_calls: Vec::new(),
            idempotent_effects: Vec::new(),
            idempotent_recovery: IdempotentRecoverySummary::default(),
            conflicts: Vec::new(),
            pending_approvals: Vec::new(),
            diagnostics: Vec::new(),
        },
        replayed_event_count: 1,
        resumed_pending_tool_count: 0,
        reused_idempotent_effect_count: 0,
        resumed: true,
        blocked: false,
        continuation: Some(RecoveryContinuation {
            summary: "readiness=ready replayed=1".to_string(),
            next_prompt: "recover this turn".to_string(),
            can_start_turn: true,
            replayed_event_count: 1,
            pending_tool_count: 0,
            reused_effect_count: 0,
            pending_approval_count: 0,
            summary_truncated: false,
            prompt_truncated: false,
        }),
    }));

    assert_eq!(
        app.reduce(TuiAction::BridgeFrame(Box::new(frame))),
        TuiEffect::None
    );
    assert!(app.pending_recovery_continuation.is_some());

    assert_eq!(
        app.reduce(TuiAction::StartRecoveryContinuation),
        TuiEffect::SubmitRecoveryContinuation {
            prompt: "recover this turn".to_string(),
            summary: "readiness=ready replayed=1".to_string(),
        }
    );
    assert!(app.pending_recovery_continuation.is_none());
}

#[test]
fn terminal_clipboard_uses_osc52_sequence() {
    let sequence = osc52_copy_sequence("hello");

    assert_eq!(sequence, "\u{1b}]52;c;aGVsbG8=\u{7}");

    let mut output = Vec::new();
    write_osc52_clipboard(&mut output, "hello").expect("write osc52");
    assert_eq!(String::from_utf8(output).expect("utf8"), sequence);
}

#[test]
fn terminal_theme_cycles_through_session_modes() {
    assert_eq!(TerminalTheme::System.name(), "system");
    assert_eq!(TerminalTheme::System.next(), TerminalTheme::Light);
    assert_eq!(TerminalTheme::Light.next(), TerminalTheme::Dark);
    assert_eq!(TerminalTheme::Dark.next(), TerminalTheme::HighContrast);
    assert_eq!(TerminalTheme::HighContrast.next(), TerminalTheme::System);
}

#[test]
fn terminal_clipboard_copy_report_writes_osc52_without_platform_dependency() {
    let mut output = Vec::new();
    let report = copy_text_to_terminal_clipboard_with_platform_result(
        &mut output,
        "hello",
        PlatformClipboardResult::default(),
    );

    assert_eq!(
        String::from_utf8(output).expect("utf8"),
        "\u{1b}]52;c;aGVsbG8=\u{7}"
    );
    assert!(report.osc52_ok);
    assert!(report.copied());
}

#[test]
fn terminal_clipboard_platform_candidates_are_environment_aware() {
    assert_eq!(
        platform_clipboard_command_candidates_for("macos", None, None)[0].display(),
        "pbcopy"
    );
    assert_eq!(
        platform_clipboard_command_candidates_for("windows", None, None)[0].display(),
        "powershell -NoProfile -Command Set-Clipboard"
    );

    let linux_wayland = platform_clipboard_command_candidates_for("linux", Some("wayland-1"), None);
    assert_eq!(linux_wayland.len(), 1);
    assert_eq!(linux_wayland[0].display(), "wl-copy");

    let linux_x11 = platform_clipboard_command_candidates_for("linux", None, Some(":0"));
    assert_eq!(
        linux_x11
            .iter()
            .map(|candidate| candidate.display())
            .collect::<Vec<_>>(),
        vec!["xclip -selection clipboard", "xsel --clipboard --input"]
    );

    assert!(platform_clipboard_command_candidates_for("linux", None, None).is_empty());
}

#[test]
fn terminal_clipboard_diagnostics_describe_fallback_path() {
    let both = ClipboardCopyReport {
        osc52_ok: true,
        platform: PlatformClipboardResult {
            attempted: true,
            command: Some("pbcopy".to_string()),
            error: None,
        },
    };
    assert_eq!(
        both.diagnostic(),
        "latest assistant output copied via OSC52 and pbcopy"
    );

    let platform_only = ClipboardCopyReport {
        osc52_ok: false,
        platform: PlatformClipboardResult {
            attempted: true,
            command: Some("wl-copy".to_string()),
            error: None,
        },
    };
    assert_eq!(
        platform_only.diagnostic(),
        "latest assistant output copied via wl-copy"
    );

    let failed = ClipboardCopyReport {
        osc52_ok: false,
        platform: PlatformClipboardResult {
            attempted: true,
            command: None,
            error: Some("pbcopy: not found".to_string()),
        },
    };
    assert_eq!(
        failed.diagnostic(),
        "terminal clipboard copy failed: pbcopy: not found"
    );
}

#[test]
fn terminal_key_mapping_routes_panel_actions_when_composer_is_empty() {
    let mut approval_app = TuiApp::default();
    approval_app
        .approval_panel
        .reduce_frame(&UiJsonFrame::Event(test_event(
            UiEventPayload::ApprovalRequested(Box::new(UiApprovalRequest {
                request_id: "00000000-0000-0000-0000-000000000001".to_string(),
                status: "pending".to_string(),
                action: "execute".to_string(),
                resource_kind: "command".to_string(),
                resource: "cargo test".to_string(),
                reason: "verify".to_string(),
                preview: UiApprovalPreviewSummary {
                    kind: "command".to_string(),
                    resource: "cargo test".to_string(),
                    summary: "Run cargo test".to_string(),
                    content_hash: None,
                    truncated: false,
                    redacted: false,
                },
                max_duration_seconds: None,
                expires_at: None,
            })),
        )));

    // Enter confirms the highlighted option, which defaults to Allow once.
    assert_eq!(
        key_event_to_action_for_app(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &approval_app
        ),
        Some(TuiAction::ApprovalConfirmSelection),
        "bare Enter confirms the highlighted option"
    );
    assert_eq!(
        approval_app.approval_selection,
        ruse_tui::ApprovalOption::AllowOnce
    );
    assert_eq!(
        key_event_to_action_for_app(
            KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
            &approval_app
        ),
        Some(TuiAction::ApprovalPanel(ApprovalPanelAction::Allow))
    );
    assert_eq!(
        key_event_to_action_for_app(
            KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
            &approval_app
        ),
        Some(TuiAction::ApprovalRejectBegin)
    );

    let mut sessions_app = TuiApp::default();
    let _ = sessions_app.session_panel.reduce(SessionPanelAction::Open);
    assert_eq!(
        key_event_to_action_for_app(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &sessions_app
        ),
        Some(TuiAction::SessionPanel(SessionPanelAction::SwitchFocused))
    );
}

#[test]
fn terminal_key_mapping_routes_slash_suggestion_controls_before_history() {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::Composer(ComposerAction::InsertText(
        "/s".to_string(),
    )));

    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(ComposerAction::SlashSuggestionNext))
    );
    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(ComposerAction::SlashSuggestionPrevious))
    );
    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(ComposerAction::SlashSuggestionComplete))
    );
    // Enter on the slash menu completes AND runs in one step (not just complete).
    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(
            ComposerAction::SlashSuggestionCompleteAndRun
        ))
    );
    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(ComposerAction::Escape))
    );

    app.reduce(TuiAction::Composer(ComposerAction::SlashSuggestionComplete));
    assert_eq!(app.composer.text(), "/status");
    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(ComposerAction::Submit))
    );

    app.reduce(TuiAction::Composer(ComposerAction::Clear));
    app.reduce(TuiAction::Composer(ComposerAction::InsertText(
        "/thinking show-all".to_string(),
    )));
    assert!(app.composer.slash_suggestions.visible());
    assert_eq!(
        key_event_to_action_for_app(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &app),
        Some(TuiAction::Composer(ComposerAction::Submit)),
        "valid command arguments must not be replaced by suggestion completion"
    );
}

#[test]
fn terminal_submit_effect_records_user_message_and_fallback_diagnostic() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::SubmitPrompt("hello agent".to_string()),
        &runtime,
        &sender,
        &mut turn,
    );
    drain_until_inactive(&mut app, &receiver, &mut turn);

    assert_eq!(app.transcript.items.len(), 2);
    assert_eq!(app.transcript.items[0].kind, TranscriptItemKind::User);
    assert_eq!(app.transcript.items[0].body, "hello agent");
    assert_eq!(app.transcript.items[1].kind, TranscriptItemKind::Diagnostic);
    assert!(app.transcript.items[1].body.contains("not configured"));
}

// Phase 5 (Cluster C): a no-op guidance note goes to the transient toast
// (neutral/dim), NOT the transcript as a red diagnostic. Only real errors are red.
#[test]
fn terminal_note_diagnostic_goes_to_toast_not_transcript() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let (sender, _receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::note("no running turn to cancel"),
        &runtime,
        &sender,
        &mut turn,
    );

    // The note surfaced as a transient toast, not a red transcript diagnostic.
    assert!(
        app.transcript.items.is_empty(),
        "a note must not enter the transcript"
    );
    let toast = app.toast.as_ref().expect("note shows a toast");
    assert_eq!(toast.message, "no running turn to cancel");
}

// A real error diagnostic renders in the transcript (red), never as a toast.
#[test]
fn terminal_error_diagnostic_goes_to_transcript_not_toast() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let (sender, _receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::error_diagnostic("something actually broke"),
        &runtime,
        &sender,
        &mut turn,
    );

    assert!(app.toast.is_none(), "a real error must not be a toast");
    assert_eq!(app.transcript.items.len(), 1);
    assert_eq!(app.transcript.items[0].kind, TranscriptItemKind::Diagnostic);
    assert!(
        app.transcript.items[0]
            .body
            .contains("something actually broke")
    );
}

#[test]
fn terminal_recovery_continuation_effect_does_not_record_internal_prompt_as_user() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::SubmitRecoveryContinuation {
            prompt: "recover this turn".to_string(),
            summary: "readiness=ready replayed=1".to_string(),
        },
        &runtime,
        &sender,
        &mut turn,
    );
    drain_until_inactive(&mut app, &receiver, &mut turn);

    assert_eq!(app.transcript.items[0].kind, TranscriptItemKind::Progress);
    assert_eq!(app.transcript.items[0].title, "recovery");
    assert_eq!(
        app.transcript
            .items
            .iter()
            .filter(|item| item.kind == TranscriptItemKind::User)
            .count(),
        0
    );
}

#[test]
fn terminal_app_state_applies_startup_frames_before_first_draw() {
    let app = build_terminal_app_state(
        ruse_tui::StatusBarState::default(),
        TerminalTheme::Dark,
        100,
        30,
        vec![
            UiJsonFrame::ConfigSummary {
                selected_provider: "freemodel".to_string(),
                selected_model: "gpt-5.5".to_string(),
                provider_ids: vec!["freemodel".to_string()],
                layer_count: 2,
                diagnostic_count: 0,
            },
            UiJsonFrame::command_retry_plan(ruse_ui_bridge::UiCommandRetryPlan::new(
                ruse_types::CommandRetryPlan {
                    command_run_id: "missing-command-evidence".to_string(),
                    attempt: ruse_types::CommandRetryAttemptLink {
                        previous_attempt_id: "missing-command-evidence".to_string(),
                        next_attempt_id: "missing-command-evidence-retry-1".to_string(),
                    },
                    readiness: ruse_types::CommandRetryReadiness::MissingEvidence,
                    classification: ruse_types::CommandRetryFailureClassification::InvalidInput,
                    command: ruse_types::CommandRetryCommandSummary {
                        display: "unknown".to_string(),
                        program: None,
                        args: Vec::new(),
                        timeout_ms: 30_000,
                        cwd: ".".to_string(),
                        classify_as_test: false,
                    },
                    exit_code: None,
                    reason: "requested from startup test".to_string(),
                    dry_run: true,
                    diagnostics: Vec::new(),
                },
            )),
        ],
    );

    assert_eq!(app.terminal_size.width, 100);
    assert_eq!(app.terminal_size.height, 30);
    assert_eq!(app.theme, TerminalTheme::Dark);
    assert_eq!(app.status.provider, "freemodel");
    assert_eq!(app.status.model, "gpt-5.5");
    let transcript = app
        .transcript
        .items
        .iter()
        .map(|item| item.render_line())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!transcript.contains("terminal app ready"));
    assert!(!transcript.contains("missing-command-evidence"));
    assert!(app.trace_panel.entries.iter().any(|entry| {
        entry.title == "command-retry" && entry.summary.contains("missing-command-evidence")
    }));
}

#[test]
fn terminal_app_state_keeps_startup_status_frames_out_of_transcript() {
    let app = build_terminal_app_state(
        ruse_tui::StatusBarState::default(),
        TerminalTheme::System,
        100,
        24,
        vec![UiJsonFrame::ConfigSummary {
            selected_provider: "freemodel".to_string(),
            selected_model: "gpt-5.5".to_string(),
            provider_ids: vec!["freemodel".to_string()],
            layer_count: 1,
            diagnostic_count: 0,
        }],
    );

    assert!(app.transcript.items.is_empty());
    let rendered = ruse_tui::render_app_to_string(&app, 100, 24).expect("render welcome");
    assert!(rendered.contains("AGENT"));
    assert!(rendered.contains("freemodel / gpt-5.5"));
    assert!(!rendered.contains("Transcript"));
}

#[test]
fn terminal_command_effects_are_routed_to_command_runtime() {
    let mut app = TuiApp::default();
    let turn_runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let command_runtime: Arc<dyn TerminalCommandRuntime> =
        Arc::new(DiagnosticTerminalCommandRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtimes(
        &mut app,
        TuiEffect::ResolveApproval {
            request_id: "00000000-0000-0000-0000-000000000001"
                .parse::<ApprovalRequestId>()
                .expect("valid approval id"),
            decision: UiApprovalDecision::Allow,
            reason: "approved from test".to_string(),
            lease_duration_seconds: None,
        },
        &turn_runtime,
        &command_runtime,
        &sender,
        &mut turn,
    );
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline
        && !app.transcript.items.iter().any(|item| {
            item.kind == TranscriptItemKind::Diagnostic
                && item.body.contains("resolve_approval is not wired")
        })
    {
        drain_terminal_frames(&mut app, &receiver, &mut turn);
        thread::sleep(Duration::from_millis(5));
    }

    assert!(app.transcript.items.iter().any(|item| {
        item.kind == TranscriptItemKind::Diagnostic
            && item.body.contains("resolve_approval is not wired")
    }));
}

#[test]
fn terminal_theme_persist_effect_is_routed_to_command_runtime() {
    let mut app = TuiApp::default();
    let turn_runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
    let command_runtime: Arc<dyn TerminalCommandRuntime> =
        Arc::new(DiagnosticTerminalCommandRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtimes(
        &mut app,
        TuiEffect::PersistTerminalTheme {
            theme: "dark".to_string(),
        },
        &turn_runtime,
        &command_runtime,
        &sender,
        &mut turn,
    );
    drain_terminal_frames(&mut app, &receiver, &mut turn);

    assert!(app.transcript.items.iter().any(|item| {
        item.kind == TranscriptItemKind::Diagnostic
            && item.body.contains("persist_terminal_theme is not wired")
    }));
}

#[test]
fn terminal_clear_transcript_action_resets_records() {
    let mut app = TuiApp::default();
    apply_terminal_effect(&mut app, TuiEffect::SubmitPrompt("hello agent".to_string()));

    app.reduce(TuiAction::ClearTranscript);

    assert!(app.transcript.items.is_empty());
}

#[test]
fn terminal_transcript_scroll_actions_update_offset() {
    let mut app = TuiApp::default();

    // Mouse-wheel steps move a few lines at a time.
    assert_eq!(app.reduce(TuiAction::ScrollTranscriptUp), TuiEffect::None);
    assert_eq!(app.transcript.scroll_offset, 3);
    assert_eq!(app.reduce(TuiAction::ScrollTranscriptDown), TuiEffect::None);
    assert_eq!(app.transcript.scroll_offset, 0);
    // Page keys jump by roughly one viewport.
    assert_eq!(
        app.reduce(TuiAction::ScrollTranscriptPageUp),
        TuiEffect::None
    );
    assert_eq!(app.transcript.scroll_offset, 18);
    assert_eq!(
        app.reduce(TuiAction::ScrollTranscriptPageDown),
        TuiEffect::None
    );
    assert_eq!(app.transcript.scroll_offset, 0);
    assert_eq!(app.reduce(TuiAction::ScrollTranscriptTop), TuiEffect::None);
    assert_eq!(app.transcript.scroll_offset, u16::MAX);
    assert_eq!(
        app.reduce(TuiAction::ScrollTranscriptBottom),
        TuiEffect::None
    );
    assert_eq!(app.transcript.scroll_offset, 0);
}

#[test]
fn terminal_cancel_requested_cancels_active_turn_and_quits_when_idle() {
    let mut app = TuiApp::default();
    let cancellation = TerminalCancellationToken::new();

    handle_cancel_turn_requested(&mut app, true, Some(&cancellation));

    assert!(cancellation.is_cancelled());
    assert!(!app.should_quit);
    assert!(
        app.transcript
            .items
            .iter()
            .any(|item| item.body.contains("cancellation requested"))
    );

    handle_cancel_turn_requested(&mut app, false, None);

    assert!(app.should_quit);
}

// Regression (Phase 1 / Cluster E): a second Ctrl+C while a turn is still
// running must escalate to a hard quit so the user is never trapped by a wedged
// tool or unresponsive provider, and the cooperative-cancel diagnostic must not
// be spammed on every press.
#[test]
fn terminal_second_cancel_force_quits_and_diagnostic_is_deduped() {
    let mut app = TuiApp::default();
    let cancellation = TerminalCancellationToken::new();

    // First Ctrl+C: cooperative cancel, one diagnostic, no quit.
    handle_cancel_turn_requested(&mut app, true, Some(&cancellation));
    assert!(cancellation.is_cancelled());
    assert!(!app.should_quit);
    let after_first = app
        .transcript
        .items
        .iter()
        .filter(|item| item.body.contains("cancellation requested"))
        .count();
    assert_eq!(
        after_first, 1,
        "first cancel records exactly one diagnostic"
    );

    // Second Ctrl+C while still active: force quit, and the cooperative-cancel
    // diagnostic is not repeated.
    handle_cancel_turn_requested(&mut app, true, Some(&cancellation));
    assert!(app.should_quit, "second cancel force-quits a wedged turn");
    let cancel_diagnostics = app
        .transcript
        .items
        .iter()
        .filter(|item| item.body.contains("cancellation requested"))
        .count();
    assert_eq!(
        cancel_diagnostics, 1,
        "cooperative-cancel diagnostic must not be spammed on repeat presses"
    );
    assert!(
        app.transcript
            .items
            .iter()
            .any(|item| item.body.contains("force quit")),
        "force quit is surfaced to the user"
    );
}

#[test]
fn terminal_cancelled_turn_drops_late_frames_until_done() {
    // After a cancel, the detached turn thread keeps running until it observes
    // the cancellation token. Its late delta frames must NOT keep streaming into
    // the transcript — cancel should feel instant — but the turn must still end
    // cleanly on the wire-level `Done` marker.
    let mut app = TuiApp::default();
    app.mark_turn_started(std::time::Instant::now());
    let mut turn = TurnController::new();
    turn.active = true;
    turn.cancellation = Some(TerminalCancellationToken::new());

    // User cancels the in-flight turn.
    handle_cancel_turn_requested(&mut app, turn.active, turn.cancellation.as_ref());
    assert!(app.cancel_requested, "cancel is armed");

    let (sender, receiver) = mpsc::channel();
    // The cancelled turn keeps emitting a late delta, then finally its Done.
    let _ = sender.send(UiJsonFrame::Event(test_event(UiEventPayload::ModelDelta {
        delta: "late output that should not appear".to_string(),
    })));
    let _ = sender.send(UiJsonFrame::Done { event_count: 1 });

    drain_terminal_frames(&mut app, &receiver, &mut turn);

    // The late delta was dropped: it never reached the transcript.
    assert!(
        !app.transcript
            .items
            .iter()
            .any(|item| item.body.contains("late output")),
        "late frames from a cancelled turn must be dropped"
    );
    // The Done frame still ended the turn.
    assert!(!turn.active, "Done ends the turn");
    assert!(!app.turn_active(), "turn clock cleared on Done");
    assert!(
        !app.cancel_requested,
        "cancel window closes when the turn ends"
    );
}

#[test]
fn approval_resolution_dispatch_does_not_block_the_ui_thread() {
    struct BlockingApprovalRuntime {
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
        finished: mpsc::Sender<()>,
    }

    impl TerminalCommandRuntime for BlockingApprovalRuntime {
        fn handle_effect(&self, effect: TuiEffect, _sender: TerminalFrameSender) {
            assert!(matches!(effect, TuiEffect::ResolveApproval { .. }));
            let _ = self.entered.send(());
            let _ = self.release.lock().expect("release lock").recv();
            let _ = self.finished.send(());
        }
    }

    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let (returned_tx, returned_rx) = mpsc::channel();
    let command_runtime: Arc<dyn TerminalCommandRuntime> = Arc::new(BlockingApprovalRuntime {
        entered: entered_tx,
        release: Mutex::new(release_rx),
        finished: finished_tx,
    });

    let dispatch = thread::spawn(move || {
        let mut app = TuiApp::default();
        let turn_runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(DiagnosticTerminalTurnRuntime);
        let (sender, _receiver) = mpsc::channel();
        let mut turn = TurnController::new();
        apply_terminal_effect_with_runtimes(
            &mut app,
            TuiEffect::ResolveApproval {
                request_id: "00000000-0000-0000-0000-000000000001"
                    .parse()
                    .expect("request id"),
                decision: UiApprovalDecision::Allow,
                reason: "approved from test".to_string(),
                lease_duration_seconds: None,
            },
            &turn_runtime,
            &command_runtime,
            &sender,
            &mut turn,
        );
        let _ = returned_tx.send(());
    });

    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("approval runtime started");
    let returned_before_release = returned_rx.recv_timeout(Duration::from_millis(100)).is_ok();
    let _ = release_tx.send(());
    dispatch.join().expect("dispatch thread");
    finished_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("approval runtime finished");

    assert!(
        returned_before_release,
        "approval continuation must run outside the UI dispatch thread"
    );
}

#[test]
fn terminal_runtime_frames_stream_into_transcript_without_duplicate_user_echo() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(FakeStreamingRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::SubmitPrompt("hello agent".to_string()),
        &runtime,
        &sender,
        &mut turn,
    );

    drain_until_inactive(&mut app, &receiver, &mut turn);

    let rendered = app
        .transcript
        .items
        .iter()
        .map(|item| item.render_line())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!turn.active);
    assert_eq!(
        app.transcript
            .items
            .iter()
            .filter(|item| item.kind == TranscriptItemKind::User)
            .count(),
        1
    );
    assert!(rendered.contains("assistant: streamed response"));
    assert!(!rendered.contains("done:"));
}

#[test]
fn terminal_session_replay_keeps_persisted_user_messages_until_done() {
    let conversation_id = ConversationId::new();
    let mut app = TuiApp {
        active_conversation_id: Some(conversation_id),
        conversation_replay_pending: true,
        ..TuiApp::default()
    };
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();
    sender
        .send(UiJsonFrame::Event(test_event_for_conversation(
            conversation_id,
            UiEventPayload::UserMessage {
                message: "persisted user message".to_string(),
            },
        )))
        .expect("send user history");
    sender
        .send(UiJsonFrame::Event(test_event_for_conversation(
            conversation_id,
            UiEventPayload::FinalResponse {
                message: "persisted assistant response".to_string(),
            },
        )))
        .expect("send assistant history");
    sender
        .send(UiJsonFrame::Done { event_count: 2 })
        .expect("send replay boundary");

    drain_terminal_frames(&mut app, &receiver, &mut turn);

    assert!(!app.conversation_replay_pending);
    assert!(app.transcript.items.iter().any(|item| {
        item.kind == TranscriptItemKind::User && item.body == "persisted user message"
    }));
    assert!(app.transcript.items.iter().any(|item| {
        item.kind == TranscriptItemKind::Model && item.body == "persisted assistant response"
    }));
}

#[test]
fn terminal_stream_replaces_startup_thinking_placeholder_with_answer() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(FakeStreamingRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::SubmitPrompt("hello agent".to_string()),
        &runtime,
        &sender,
        &mut turn,
    );
    drain_until_inactive(&mut app, &receiver, &mut turn);

    assert!(
        !app.transcript
            .items
            .iter()
            .any(|item| { item.kind == TranscriptItemKind::Reasoning && item.body == "thinking" })
    );
    assert!(app.transcript.items.iter().any(|item| {
        item.kind == TranscriptItemKind::Model && item.body == "streamed response"
    }));
}

#[test]
fn terminal_turn_snapshots_reasoning_effort_for_direct_and_queued_prompts() {
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(RecordingOptionsRuntime {
        recorded: Arc::clone(&recorded),
    });
    let (sender, receiver) = mpsc::channel();
    let mut app = TuiApp::default();
    app.status.reasoning_effort = ReasoningEffort::Low;
    let mut turn = TurnController::new();

    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::SubmitPrompt("direct".to_string()),
        &runtime,
        &sender,
        &mut turn,
    );
    app.status.reasoning_effort = ReasoningEffort::High;
    drain_until_inactive(&mut app, &receiver, &mut turn);

    assert_eq!(
        recorded.lock().expect("recorded options").as_slice(),
        &[ReasoningEffort::Low]
    );

    turn.active = true;
    app.status.reasoning_effort = ReasoningEffort::Medium;
    apply_terminal_effect_with_runtime(
        &mut app,
        TuiEffect::SubmitPrompt("queued".to_string()),
        &runtime,
        &sender,
        &mut turn,
    );
    app.status.reasoning_effort = ReasoningEffort::XHigh;

    assert_eq!(
        turn.queued_prompts
            .front()
            .expect("queued prompt")
            .options
            .reasoning_effort,
        ReasoningEffort::Medium
    );
}

#[test]
fn terminal_submit_queues_prompts_while_turn_is_active() {
    let mut app = TuiApp::default();
    let runtime: Arc<dyn TerminalTurnRuntime> = Arc::new(FakeStreamingRuntime);
    let command_runtime: Arc<dyn TerminalCommandRuntime> =
        Arc::new(DiagnosticTerminalCommandRuntime);
    let (sender, receiver) = mpsc::channel();
    let mut turn = TurnController::new();

    ruse_tui::apply_terminal_effect_with_runtime_and_queue(
        &mut app,
        TuiEffect::SubmitPrompt("first".to_string()),
        TerminalEffectRuntime {
            turn_runtime: &runtime,
            command_runtime: &command_runtime,
            sender: &sender,
            turn: &mut turn,
        },
    );
    ruse_tui::apply_terminal_effect_with_runtime_and_queue(
        &mut app,
        TuiEffect::SubmitPrompt("second".to_string()),
        TerminalEffectRuntime {
            turn_runtime: &runtime,
            command_runtime: &command_runtime,
            sender: &sender,
            turn: &mut turn,
        },
    );

    assert!(turn.active);
    assert_eq!(turn.queued_prompts.len(), 1);
    assert!(
        app.transcript
            .items
            .iter()
            .any(|item| item.title == "user queued" && item.body == "second")
    );

    drain_until_inactive(&mut app, &receiver, &mut turn);
    start_next_queued_prompt(&mut app, &runtime, &sender, &mut turn);
    drain_until_inactive(&mut app, &receiver, &mut turn);

    assert!(!turn.active);
    assert!(turn.queued_prompts.is_empty());
    assert!(
        app.transcript
            .items
            .iter()
            .any(|item| item.title == "queue" && item.body == "starting queued prompt")
    );
    assert_eq!(
        app.transcript
            .items
            .iter()
            .filter(|item| item.kind == TranscriptItemKind::User)
            .count(),
        2
    );
}

fn drain_until_inactive(
    app: &mut TuiApp,
    receiver: &mpsc::Receiver<UiJsonFrame>,
    turn: &mut TurnController,
) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while turn.active && Instant::now() < deadline {
        drain_terminal_frames(app, receiver, turn);
        thread::sleep(Duration::from_millis(5));
    }
    drain_terminal_frames(app, receiver, turn);
}

struct RecordingOptionsRuntime {
    recorded: Arc<Mutex<Vec<ReasoningEffort>>>,
}

impl TerminalTurnRuntime for RecordingOptionsRuntime {
    fn run_turn(
        &self,
        _prompt: String,
        sender: TerminalFrameSender,
        _cancellation: TerminalCancellationToken,
    ) {
        let _ = sender.send(UiJsonFrame::Done { event_count: 0 });
    }

    fn run_turn_with_options(
        &self,
        prompt: String,
        sender: TerminalFrameSender,
        cancellation: TerminalCancellationToken,
        options: TerminalTurnOptions,
    ) {
        self.recorded
            .lock()
            .expect("record options")
            .push(options.reasoning_effort);
        self.run_turn(prompt, sender, cancellation);
    }
}

struct FakeStreamingRuntime;

impl TerminalTurnRuntime for FakeStreamingRuntime {
    fn run_turn(
        &self,
        prompt: String,
        sender: TerminalFrameSender,
        _cancellation: TerminalCancellationToken,
    ) {
        let _ = sender.send(UiJsonFrame::Event(test_event(
            UiEventPayload::UserMessage { message: prompt },
        )));
        let _ = sender.send(UiJsonFrame::Event(test_event(UiEventPayload::ModelDelta {
            delta: "streamed ".to_string(),
        })));
        let _ = sender.send(UiJsonFrame::Event(test_event(
            UiEventPayload::FinalResponse {
                message: "streamed response".to_string(),
            },
        )));
        let _ = sender.send(UiJsonFrame::Done { event_count: 3 });
    }
}

fn test_event(payload: UiEventPayload) -> UiEventEnvelope {
    test_event_for_conversation(ConversationId::new(), payload)
}

fn test_event_for_conversation(
    conversation_id: ConversationId,
    payload: UiEventPayload,
) -> UiEventEnvelope {
    UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        conversation_id,
        turn_id: Some(TurnId::new()),
        task_id: None::<TaskId>,
        correlation_id: CorrelationId::new(),
        payload,
    }
}
