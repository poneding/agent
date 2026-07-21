use ruse_tui::{TranscriptItem, TranscriptItemKind, TuiApp, render_app_to_string};

#[test]
fn transcript_render_draws_active_stream() {
    let mut app = TuiApp::default();
    app.transcript
        .push(TranscriptItem::streaming_model("partial answer"));

    let rendered = render_app_to_string(&app, 88, 20).expect("render app");

    assert!(rendered.contains("partial answer"));
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
    assert!(rendered.contains("call-1 completed") || rendered.contains("tool result"));
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

    assert!(rendered.contains("runtime.error"));
    assert!(rendered.contains("connection dropped"));
}

#[test]
fn transcript_render_model_markdown_headings_and_code() {
    let mut app = TuiApp::default();
    app.markdown_enabled = true;
    app.transcript.push(TranscriptItem::completed_model(
        "# Title\n\nUse `code` and **bold**.\n\n```rust\nfn main() {}\n```\n",
    ));

    let rendered = render_app_to_string(&app, 88, 24).expect("render app");

    assert!(rendered.contains("Title"));
    assert!(rendered.contains("code"));
    assert!(rendered.contains("bold"));
    assert!(rendered.contains("```") || rendered.contains("fn main"));
}

#[test]
fn transcript_render_plain_markdown_fallback() {
    let mut app = TuiApp::default();
    app.markdown_enabled = false;
    app.transcript
        .push(TranscriptItem::completed_model("**still literal**"));

    let rendered = render_app_to_string(&app, 88, 20).expect("render app");

    assert!(rendered.contains("**still literal**"));
}

#[test]
fn transcript_render_reasoning_markdown_when_expanded() {
    let mut app = TuiApp::default();
    app.markdown_enabled = true;
    let mut item = TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "Consider **tradeoffs** carefully.",
        false,
    );
    item.set_reasoning_collapsed(false);
    app.transcript.push(item);

    let rendered = render_app_to_string(&app, 88, 24).expect("render app");

    assert!(rendered.contains("Thought") || rendered.contains("tradeoffs"));
    assert!(rendered.contains("tradeoffs"));
}
