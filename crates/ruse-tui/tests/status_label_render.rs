//! Status-label visual language: transcript blocks and the activity line no longer
//! use inverted chip backgrounds. Labels render as `• word` with a status color,
//! tense-aware wording (Thinking… / Thought), and a two-tone marquee while live.

use ruse_tui::{TranscriptItem, TranscriptItemKind, TuiApp, render_app_to_string};

#[test]
fn reasoning_and_tool_labels_use_bullet_without_chip_background() {
    let mut app = TuiApp::default();
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "reasoning",
        "some plan",
        true, // streaming → Thinking…
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "reasoning",
        "done plan",
        false, // settled → Thought
    ));
    app.transcript.push(TranscriptItem::tool_call(
        "c1",
        "workspace.write_file",
        "running",
        "write file src/main.rs",
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Diagnostic,
        "runtime.error",
        "connection dropped",
        false,
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Approval,
        "approval",
        "allow shell",
        false,
    ));
    app.mark_turn_started(std::time::Instant::now());
    app.spinner_frame = 2;
    app.elapsed_seconds = 3;

    let rendered = render_app_to_string(&app, 100, 28).expect("render");

    assert!(
        rendered.contains("• Thinking…"),
        "streaming reasoning must use progressive tense: {rendered}"
    );
    assert!(
        rendered.contains("• Thought"),
        "finished reasoning must use past tense: {rendered}"
    );
    assert!(
        rendered.contains("• write_file"),
        "tool labels must use a bullet prefix, not a chip: {rendered}"
    );
    assert!(
        rendered.contains("• diagnostic"),
        "diagnostics must use a bullet prefix: {rendered}"
    );
    assert!(
        rendered.contains("• approval"),
        "approvals must use a bullet prefix: {rendered}"
    );
    // No inverted chip padding spaces like " thinking " / " write_file ".
    assert!(
        !rendered.contains(" thinking "),
        "old chip-style thinking label must be gone: {rendered}"
    );
    assert!(
        rendered.contains("Working…")
            || rendered.contains("Thinking…")
            || rendered.contains("Responding…"),
        "activity line must show a progressive-tense verb: {rendered}"
    );
}
