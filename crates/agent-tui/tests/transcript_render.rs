use agent_tui::{TranscriptItem, TranscriptItemKind, TuiApp, render_app_to_string};

#[test]
fn transcript_render_draws_active_stream() {
    let mut app = TuiApp::default();
    app.transcript
        .push(TranscriptItem::streaming_model("partial answer"));

    let rendered = render_app_to_string(&app, 88, 20).expect("render app");

    assert!(rendered.contains("Transcript"));
    assert!(rendered.contains("assistant"));
    assert!(rendered.contains("partial answer"));
    assert!(rendered.contains("streaming"));
}

#[test]
fn transcript_render_draws_completed_response_and_tool_summary() {
    let mut app = TuiApp::default();
    app.transcript
        .push(TranscriptItem::completed_model("final answer"));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool result",
        "call-1 completed",
        false,
    ));

    let rendered = render_app_to_string(&app, 88, 20).expect("render app");

    assert!(rendered.contains("final answer"));
    assert!(rendered.contains("tool result"));
    assert!(rendered.contains("call-1 completed"));
}

#[test]
fn transcript_render_draws_diagnostics_in_narrow_layout() {
    let mut app = TuiApp::default();
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Diagnostic,
        "runtime.error",
        "connection dropped",
        false,
    ));

    let rendered = render_app_to_string(&app, 44, 14).expect("render app");

    assert!(rendered.contains("Transcript"));
    assert!(rendered.contains("runtime.error"));
    assert!(rendered.contains("connection dropped"));
    assert!(!rendered.contains("Panel"));
}
