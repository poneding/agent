//! Status-label visual language: transcript blocks and the activity line no longer
//! use inverted chip backgrounds. Live labels use the star spinner prefix
//! (`✶/✸/✹/…`); settled labels keep a solid `•`. Tense-aware wording
//! (Thinking... / Thought, Running / Ran); Thinking may also marquee its letters.
//! Thinking headers show live `[Ns · N tokens]` stats plus a click/ctrl+p
//! expand-collapse hint; hover brightens the whole header.

use ruse_tui::{TranscriptItem, TranscriptItemKind, TuiApp, render_app_to_string};

#[test]
fn reasoning_and_tool_labels_use_bullet_without_chip_background() {
    let mut app = TuiApp::default();
    let mut streaming = TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "reasoning",
        "some plan about the tradeoffs",
        true, // streaming → Thinking...
    );
    streaming.thinking_seconds = 3;
    streaming.thinking_tokens = 100;
    app.transcript.push(streaming);
    let mut done = TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "reasoning",
        "done plan about the tradeoffs",
        false, // settled → Thought
    );
    done.thinking_seconds = 3;
    done.thinking_tokens = 100;
    app.transcript.push(done);
    app.transcript.push(TranscriptItem::tool_call(
        "c1",
        "workspace.write_file",
        "running",
        "write file src/main.rs",
    ));
    app.transcript.push(TranscriptItem::tool_call(
        "c2",
        "workspace.shell_command",
        "running",
        "run cargo test -q",
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "shell cargo test -q\npassed",
        false,
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
        rendered.contains("Thinking...") && !rendered.contains("• Thinking..."),
        "streaming reasoning must use spinner prefix + progressive tense: {rendered}"
    );
    assert!(
        !rendered.contains("Thinking…"),
        "progressive tense must not use unicode ellipsis: {rendered}"
    );
    assert!(
        rendered.contains("• Thought"),
        "finished reasoning must use past tense: {rendered}"
    );
    assert!(
        rendered.contains("[3s · 100 tokens]"),
        "thinking headers must show live duration and tokens: {rendered}"
    );
    assert!(
        !rendered.contains("[0s · 0 tokens]"),
        "empty thinking stats must not be painted: {rendered}"
    );
    assert!(
        rendered.contains("click or ctrl+p to expand"),
        "collapsed thinking must hint click/ctrl+p to expand: {rendered}"
    );
    assert!(
        !rendered.contains("▸") && !rendered.contains("▾"),
        "thinking headers must not show triangle expand markers: {rendered}"
    );
    assert!(
        rendered.contains("Writing...")
            && !rendered.contains("• Writing...")
            && rendered.contains("src/main.rs"),
        "write_file must use spinner + Writing... + path: {rendered}"
    );
    assert!(
        !rendered.contains(" write_file") && !rendered.contains("• write_file"),
        "raw tool ids must not appear as status labels: {rendered}"
    );
    assert!(
        rendered.contains("Running...")
            && !rendered.contains("• Running...")
            && rendered.contains("cargo test -q"),
        "running shell must show spinner + Running... <command>: {rendered}"
    );
    assert!(
        rendered.contains("• Ran") && rendered.contains("cargo test -q"),
        "finished shell must show Ran <command>: {rendered}"
    );
    // Finished shell auto-expands so the user sees the command result.
    assert!(
        rendered.contains("passed"),
        "finished shell must show command output by default: {rendered}"
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
        rendered.contains("Working...")
            || rendered.contains("Thinking...")
            || rendered.contains("Responding..."),
        "activity line must show a progressive-tense verb with ASCII ...: {rendered}"
    );
}

#[test]
fn expanded_thought_shows_collapse_hint_with_stats() {
    let mut app = TuiApp::default();
    let mut item = TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "Consider the tradeoffs carefully and choose the safer path.",
        false,
    );
    item.thinking_seconds = 12;
    item.thinking_tokens = 42;
    item.set_reasoning_collapsed(false);
    app.transcript.push(item);

    let rendered = render_app_to_string(&app, 120, 24).expect("render");
    assert!(
        rendered.contains("• Thought")
            && rendered.contains("[12s · 42 tokens]")
            && rendered.contains("click or ctrl+p to collapse"),
        "expanded Thought must keep stats and collapse hint: {rendered}"
    );
    assert!(
        rendered.contains("tradeoffs"),
        "expanded body must be visible: {rendered}"
    );
}

#[test]
fn expanded_thought_preserves_approval_segment_line_breaks() {
    let mut app = TuiApp::default();
    let mut item = TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "before approval  \nafter approval",
        false,
    );
    item.set_reasoning_collapsed(false);
    app.transcript.push(item);

    let rendered = render_app_to_string(&app, 120, 24).expect("render");
    let lines = rendered.lines().collect::<Vec<_>>();
    let before = lines
        .iter()
        .position(|line| line.contains("before approval"))
        .expect("first reasoning segment");
    let after = lines
        .iter()
        .position(|line| line.contains("after approval"))
        .expect("resumed reasoning segment");

    assert_eq!(
        after,
        before + 1,
        "approval-resumed reasoning must start on the next terminal row: {rendered}"
    );
}

#[test]
fn shell_waiting_running_ran_labels() {
    let mut app = TuiApp::default();
    // Proposed / awaiting approval.
    app.transcript.push(TranscriptItem::tool_call(
        "s0",
        "workspace.shell_command",
        "proposed",
        "run ls -la",
    ));
    // Actively executing.
    app.transcript.push(TranscriptItem::tool_call(
        "s1",
        "workspace.shell_command",
        "running",
        "run ls -la",
    ));
    // Finished with user-facing stdout only.
    let mut done = TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "shell ls -la\ntotal 48\ndrwxr-xr-x  5 dp  staff  160 .",
        false,
    );
    done.tool_output_expanded = true;
    app.transcript.push(done);

    let rendered = render_app_to_string(&app, 100, 24).expect("render");
    assert!(
        rendered.contains("Waiting...")
            && !rendered.contains("• Waiting...")
            && rendered.contains("ls -la"),
        "proposed shell must show spinner + Waiting...: {rendered}"
    );
    assert!(
        rendered.contains("Running...")
            && !rendered.contains("• Running...")
            && rendered.contains("ls -la"),
        "active shell must show spinner + Running...: {rendered}"
    );
    assert!(
        rendered.contains("• Ran") && rendered.contains("total 48"),
        "finished shell must show Ran + stdout: {rendered}"
    );
    let total_row = rendered
        .lines()
        .position(|line| line.contains("total 48"))
        .expect("total row");
    let entry_row = rendered
        .lines()
        .position(|line| line.contains("drwxr-xr-x"))
        .expect("directory row");
    assert_ne!(
        total_row, entry_row,
        "each stdout newline must render as a separate terminal row: {rendered}"
    );
    assert!(
        !rendered.contains("sandbox_profile"),
        "raw shell JSON must not appear: {rendered}"
    );
}

#[test]
fn write_and_read_tools_use_tense_labels_with_muted_path() {
    let mut app = TuiApp::default();
    app.transcript.push(TranscriptItem::tool_call(
        "w1",
        "workspace.write_file",
        "running",
        "write file src/lib.rs",
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "write_file src/lib.rs\nsrc/lib.rs (12 bytes)",
        false,
    ));
    app.transcript.push(TranscriptItem::tool_call(
        "r1",
        "workspace.read_file",
        "running",
        "read file crates/foo.rs",
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "read_file crates/foo.rs\ncrates/foo.rs",
        false,
    ));

    let rendered = render_app_to_string(&app, 100, 24).expect("render");
    assert!(
        rendered.contains("Writing...")
            && !rendered.contains("• Writing...")
            && rendered.contains("src/lib.rs"),
        "live write must say spinner + Writing... path: {rendered}"
    );
    assert!(
        rendered.contains("• Wrote") && rendered.contains("src/lib.rs"),
        "finished write must say Wrote path: {rendered}"
    );
    assert!(
        rendered.contains("Reading...")
            && !rendered.contains("• Reading...")
            && rendered.contains("crates/foo.rs"),
        "live read must say spinner + Reading... path: {rendered}"
    );
    assert!(
        rendered.contains("• Read") && rendered.contains("crates/foo.rs"),
        "finished read must say Read path: {rendered}"
    );
}

#[test]
fn edit_tool_expands_diff_by_default() {
    let mut app = TuiApp::default();
    let mut item = TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "edit_file src/main.rs\n--- src/main.rs\n+++ src/main.rs\n@@\n-old line\n+new line",
        false,
    );
    // Auto-expand is set by the transcript reducer; simulate that here.
    item.tool_output_expanded = true;
    app.transcript.push(item);

    let rendered = render_app_to_string(&app, 100, 24).expect("render");
    assert!(
        rendered.contains("• Edited") && rendered.contains("src/main.rs"),
        "edit result must use Edited + path: {rendered}"
    );
    assert!(
        rendered.contains("old line") && rendered.contains("new line"),
        "edit diff body must be visible by default: {rendered}"
    );
    // Guttered layout: `1 - old line` / `1 + new line` (not raw `-old`).
    assert!(
        rendered.contains("-") && rendered.contains("+"),
        "edit diff must show add/remove markers: {rendered}"
    );
    assert!(
        !rendered.contains("+N lines"),
        "expanded edit must not show a collapse overflow hint: {rendered}"
    );
}

#[test]
fn edit_diff_side_by_side_when_wide() {
    let mut app = TuiApp::default();
    let mut item = TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "write_file src/main.rs\n--- src/main.rs\n+++ src/main.rs\n@@ -1 +1 @@\n-old line\n+new line",
        false,
    );
    item.tool_output_expanded = true;
    app.transcript.push(item);

    // Wide enough for side-by-side (threshold 100 content cols + gutters).
    let rendered = render_app_to_string(&app, 140, 24).expect("render");
    assert!(
        rendered.contains("• Wrote") && rendered.contains("src/main.rs"),
        "write result header: {rendered}"
    );
    assert!(
        rendered.contains('│'),
        "wide terminal must use side-by-side column divider: {rendered}"
    );
    assert!(
        rendered.contains("old line") && rendered.contains("new line"),
        "both sides of the edit must be visible: {rendered}"
    );
}

#[test]
fn edit_diff_unified_when_narrow() {
    let mut app = TuiApp::default();
    let mut item = TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "edit_file src/main.rs\n@@ -1 +1 @@\n-old line\n+new line",
        false,
    );
    item.tool_output_expanded = true;
    app.transcript.push(item);

    let rendered = render_app_to_string(&app, 60, 20).expect("render");
    assert!(
        !rendered.contains('│'),
        "narrow terminal must stay unified (no column divider): {rendered}"
    );
    assert!(
        rendered.contains("old line") && rendered.contains("new line"),
        "unified body must still show both lines: {rendered}"
    );
}

#[test]
fn empty_settled_thought_is_not_rendered() {
    let mut app = TuiApp::default();
    // Ghost item that would previously paint as "Thought [0s · 0 tokens]".
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "thinking",
        false,
    ));
    app.transcript
        .push(TranscriptItem::completed_model("real answer"));

    let rendered = render_app_to_string(&app, 100, 20).expect("render");
    assert!(
        !rendered.contains("Thought"),
        "empty settled Thought must be suppressed: {rendered}"
    );
    assert!(
        !rendered.contains("[0s · 0 tokens]"),
        "zero stats must not appear for empty reasoning: {rendered}"
    );
    assert!(
        rendered.contains("real answer"),
        "model answer must still render: {rendered}"
    );
}

#[test]
fn expanded_shell_shows_muted_output_detail() {
    let mut app = TuiApp::default();
    let mut item = TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool",
        "shell cargo test -q\nline one\nline two\nline three",
        false,
    );
    item.tool_output_expanded = true;
    app.transcript.push(item);

    let rendered = render_app_to_string(&app, 100, 24).expect("render");
    assert!(
        rendered.contains("• Ran") && rendered.contains("cargo test -q"),
        "expanded shell still shows Ran <command>: {rendered}"
    );
    assert!(
        rendered.contains("line one") && rendered.contains("line two"),
        "expanded shell must reveal command output: {rendered}"
    );
}
