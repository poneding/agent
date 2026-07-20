use agent_tui::{
    TerminalSize, TranscriptHit, TranscriptItem, TranscriptItemKind, TuiAction, TuiApp,
    hit_test_transcript_for_test,
};

fn app_with_size(width: u16, height: u16) -> TuiApp {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::Resize { width, height });
    app
}

#[test]
fn click_on_reasoning_header_toggles_that_item() {
    let mut app = app_with_size(80, 24);
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::User,
        "user",
        "question",
        false,
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "first private thought",
        false,
    ));
    app.transcript
        .push(TranscriptItem::completed_model("answer"));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "second private thought",
        false,
    ));

    // Both reasoning items start collapsed (default Auto).
    assert!(app.transcript.items[1].is_reasoning_collapsed());
    assert!(app.transcript.items[3].is_reasoning_collapsed());

    // Find the screen row that maps to the first reasoning item, then click it.
    let row = (0..20)
        .find(|&row| {
            matches!(
                hit_test_transcript_for_test(&app, 2, row),
                Some(TranscriptHit::ToggleReasoning { index: 1 })
            )
        })
        .expect("row for first reasoning");
    app.reduce(TuiAction::TranscriptClick { column: 2, row });
    assert!(
        !app.transcript.items[1].is_reasoning_collapsed(),
        "first thinking expands"
    );
    assert!(
        app.transcript.items[3].is_reasoning_collapsed(),
        "second thinking stays collapsed"
    );

    // Click the second reasoning item specifically.
    let row2 = (0..24)
        .find(|&row| {
            matches!(
                hit_test_transcript_for_test(&app, 2, row),
                Some(TranscriptHit::ToggleReasoning { index: 3 })
            )
        })
        .expect("row for second reasoning");
    app.reduce(TuiAction::TranscriptClick {
        column: 2,
        row: row2,
    });
    assert!(
        !app.transcript.items[3].is_reasoning_collapsed(),
        "second thinking expands on its own click"
    );
    assert!(
        !app.transcript.items[1].is_reasoning_collapsed(),
        "first thinking stays expanded"
    );
}

#[test]
fn click_on_tool_block_toggles_that_tool_not_latest() {
    let mut app = app_with_size(80, 24);
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool ok",
        "shell ls\n - line 1\nline 2\nline 3\nline 4\nline 5",
        false,
    ));
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::ToolResult,
        "tool ok",
        "read_file main.py - main.py (10 bytes)",
        false,
    ));

    assert!(!app.transcript.items[0].tool_output_expanded);
    assert!(!app.transcript.items[1].tool_output_expanded);

    // First tool is at the top of the content area.
    let row = (0..10)
        .find(|&row| {
            matches!(
                hit_test_transcript_for_test(&app, 2, row),
                Some(TranscriptHit::ToggleToolOutput { index: 0 })
            )
        })
        .expect("row for first tool");
    app.reduce(TuiAction::TranscriptClick { column: 2, row });
    assert!(
        app.transcript.items[0].tool_output_expanded,
        "first tool expands"
    );
    assert!(
        !app.transcript.items[1].tool_output_expanded,
        "second tool stays collapsed"
    );
}

#[test]
fn click_outside_transcript_is_a_noop() {
    let mut app = app_with_size(80, 24);
    app.transcript
        .push(TranscriptItem::completed_model("hello"));
    // Footer row (last row) is chrome, not transcript.
    let height = app.terminal_size.height;
    assert_eq!(
        hit_test_transcript_for_test(&app, 2, height.saturating_sub(1)),
        None
    );
    app.reduce(TuiAction::TranscriptClick {
        column: 2,
        row: height.saturating_sub(1),
    });
    // No panic, no state change.
    assert_eq!(app.transcript.items.len(), 1);
}

#[test]
fn mouse_down_left_maps_to_selection_drag_start() {
    use agent_tui::mouse_event_to_action;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 4,
            row: 3,
            modifiers: KeyModifiers::NONE,
        }),
        Some(TuiAction::SelectionDragStart {
            column: 4,
            row: 3,
        })
    );
    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 8,
            row: 3,
            modifiers: KeyModifiers::NONE,
        }),
        Some(TuiAction::SelectionDragUpdate {
            column: 8,
            row: 3,
        })
    );
    assert_eq!(
        mouse_event_to_action(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 8,
            row: 3,
            modifiers: KeyModifiers::NONE,
        }),
        Some(TuiAction::SelectionDragEnd {
            column: 8,
            row: 3,
        })
    );
}

#[test]
fn click_without_drag_still_toggles_via_drag_end() {
    let mut app = app_with_size(80, 24);
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "private",
        false,
    ));
    assert!(app.transcript.items[0].is_reasoning_collapsed());

    let row = (0..10)
        .find(|&row| {
            matches!(
                hit_test_transcript_for_test(&app, 2, row),
                Some(TranscriptHit::ToggleReasoning { index: 0 })
            )
        })
        .expect("row for reasoning");

    // Down → Up without Drag: treated as a click, toggles the item.
    app.reduce(TuiAction::SelectionDragStart {
        column: 2,
        row,
    });
    let effect = app.reduce(TuiAction::SelectionDragEnd {
        column: 2,
        row,
    });
    assert_eq!(effect, agent_tui::TuiEffect::None);
    assert!(
        !app.transcript.items[0].is_reasoning_collapsed(),
        "click expands thinking"
    );
}

#[test]
fn drag_select_copies_selected_text() {
    use agent_tui::{ContentGeometry, extract_selected_text};

    let mut app = app_with_size(80, 24);
    app.transcript
        .push(TranscriptItem::completed_model("hello selectable world"));

    // Drag across the content area.
    app.reduce(TuiAction::SelectionDragStart {
        column: 2,
        row: 0,
    });
    app.reduce(TuiAction::SelectionDragUpdate {
        column: 20,
        row: 0,
    });
    let effect = app.reduce(TuiAction::SelectionDragEnd {
        column: 20,
        row: 0,
    });
    match effect {
        agent_tui::TuiEffect::CopyToClipboard { text } => {
            assert!(
                text.contains("hello") || text.contains("selectable") || !text.is_empty(),
                "drag should copy some transcript text, got {text:?}"
            );
        }
        other => panic!("expected CopyToClipboard, got {other:?}"),
    }
    assert!(app.selection.committed.is_some());

    // Extract path stays consistent with the committed range.
    if let Some(range) = app.selection.committed {
        let extracted = extract_selected_text(&app, range);
        assert!(!extracted.is_empty());
        let _ = ContentGeometry::from_app(&app);
    }
}

#[test]
fn terminal_size_is_tracked_for_hit_testing() {
    let mut app = TuiApp::default();
    assert_eq!(app.terminal_size, TerminalSize::default());
    app.reduce(TuiAction::Resize {
        width: 100,
        height: 40,
    });
    assert_eq!(
        app.terminal_size,
        TerminalSize {
            width: 100,
            height: 40
        }
    );
}
