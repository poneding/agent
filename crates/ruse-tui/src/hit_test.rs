//! Transcript click hit-testing.
//!
//! Rebuilds the same line layout the renderer uses (item lines → wrap → pad →
//! blank-run collapse → scroll window) and maps each painted row back to the
//! originating transcript item. A left-click on a reasoning header or tool
//! block then toggles that specific item instead of "the latest one".

use ratatui::text::Line;

use crate::render::{
    collapse_visually_blank_runs_for_hit, pad_block_backgrounds_for_hit,
    transcript_item_lines_for_hit, transcript_visible_start, wrap_transcript_lines,
};
use crate::{TerminalTheme, TranscriptItem, TranscriptItemKind, TranscriptState, TuiApp};

/// What a click on a transcript row should do.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TranscriptHit {
    /// Toggle reasoning visibility for `items[index]`.
    ToggleReasoning { index: usize },
    /// Toggle tool-output expansion for `items[index]`.
    ToggleToolOutput { index: usize },
}

/// Map a terminal-absolute click `(column, row)` to a collapsible transcript
/// item, if the click lands on one. Returns `None` for empty space, user/model
/// prose, separators, chrome, or when the click is outside the transcript area.
#[must_use]
pub fn hit_test_transcript(app: &TuiApp, column: u16, row: u16) -> Option<TranscriptHit> {
    let layout = transcript_layout_for_hit(app)?;
    if row < layout.content_y || row >= layout.content_y.saturating_add(layout.content_height) {
        return None;
    }
    if column < layout.content_x
        || column >= layout.content_x.saturating_add(layout.content_width)
    {
        return None;
    }
    let local_row = usize::from(row.saturating_sub(layout.content_y));
    let item_index = (*layout.row_to_item.get(local_row)?)?;
    let item = app.transcript.items.get(item_index)?;
    match item.kind {
        TranscriptItemKind::Reasoning => Some(TranscriptHit::ToggleReasoning { index: item_index }),
        TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => {
            Some(TranscriptHit::ToggleToolOutput { index: item_index })
        }
        _ => None,
    }
}

struct TranscriptHitLayout {
    content_x: u16,
    content_y: u16,
    content_width: u16,
    content_height: u16,
    /// For each visible screen row inside the content area, the source item
    /// index (or none for pure separators / empty pad). Length == content_height
    /// when the transcript is non-empty; shorter only if content is empty.
    row_to_item: Vec<Option<usize>>,
}

fn transcript_layout_for_hit(app: &TuiApp) -> Option<TranscriptHitLayout> {
    if app.transcript.items.is_empty() {
        return None;
    }
    let (content_x, content_y, content_width, content_height) = primary_content_rect(app)?;
    if content_width == 0 || content_height == 0 {
        return None;
    }

    let theme = app.theme;
    let (lines, owners) = build_owned_lines(&app.transcript.items, theme);
    let wrapped = wrap_with_owners(lines, owners, content_width);
    let padded = pad_block_backgrounds_for_hit(wrapped, content_width);
    let collapsed = collapse_visually_blank_runs_for_hit(padded);

    let total = collapsed.len();
    let visible_height = usize::from(content_height);
    let start = transcript_visible_start(total, visible_height, app.transcript.scroll_offset);
    let end = (start + visible_height).min(total);

    let mut row_to_item = Vec::with_capacity(visible_height);
    for entry in collapsed.iter().take(end).skip(start) {
        row_to_item.push(entry.1);
    }
    // If the transcript is shorter than the viewport, trailing empty rows have
    // no owner — pad so local_row indexing stays in range.
    while row_to_item.len() < visible_height {
        row_to_item.push(None);
    }

    Some(TranscriptHitLayout {
        content_x,
        content_y,
        content_width,
        content_height,
        row_to_item,
    })
}

/// Approximate the primary transcript content rect from the same constraints
/// `render_chat` uses. Side panels are gone, so the primary area is full width.
fn primary_content_rect(app: &TuiApp) -> Option<(u16, u16, u16, u16)> {
    let width = app.terminal_size.width;
    let height = app.terminal_size.height;
    if width == 0 || height == 0 {
        return None;
    }

    let suggestion_height = slash_suggestion_height_for_hit(&app.composer.slash_suggestions);
    let activity_height: u16 = if app.turn_state().is_active() { 2 } else { 0 };
    let composer_height = composer_height_for_hit(app);
    let footer_height: u16 = 1;

    let reserved = activity_height
        .saturating_add(suggestion_height)
        .saturating_add(composer_height)
        .saturating_add(footer_height);
    let primary_height = height.saturating_sub(reserved).max(1);
    // Primary area starts at y=0 in the chat layout.
    let gutter = u16::from(width >= 4);
    let content_x = gutter;
    let content_y = 0u16;
    let content_width = width.saturating_sub(gutter.saturating_mul(2));
    let content_height = primary_height;
    Some((content_x, content_y, content_width, content_height))
}

fn slash_suggestion_height_for_hit(suggestions: &crate::SlashSuggestionState) -> u16 {
    if !suggestions.visible() {
        return 0;
    }
    if suggestions.matches.is_empty() {
        1
    } else {
        u16::try_from(suggestions.matches.len().min(7)).unwrap_or(7)
    }
}

fn composer_height_for_hit(app: &TuiApp) -> u16 {
    let text_rows = u16::try_from(app.composer.line_count())
        .unwrap_or(6)
        .clamp(1, 6);
    text_rows.saturating_add(2)
}

fn build_owned_lines(
    items: &[TranscriptItem],
    theme: TerminalTheme,
) -> (Vec<Line<'static>>, Vec<Option<usize>>) {
    let mut lines = Vec::new();
    let mut owners = Vec::new();
    let mut rendered_any = false;
    for (index, item) in items.iter().enumerate() {
        let item_lines = transcript_item_lines_for_hit(item, theme);
        if item_lines.is_empty() {
            continue;
        }
        if rendered_any {
            lines.push(Line::from(""));
            owners.push(None);
        }
        rendered_any = true;
        for line in item_lines {
            lines.push(line);
            owners.push(Some(index));
        }
    }
    (lines, owners)
}

fn wrap_with_owners(
    lines: Vec<Line<'static>>,
    owners: Vec<Option<usize>>,
    width: u16,
) -> Vec<(Line<'static>, Option<usize>)> {
    // wrap_transcript_lines expands each logical line into 1+ wrapped rows.
    // We re-wrap per line so ownership is preserved across wraps.
    let width = width.max(1);
    let mut out = Vec::new();
    for (line, owner) in lines.into_iter().zip(owners.into_iter()) {
        let wrapped = wrap_transcript_lines(vec![line], width);
        for row in wrapped {
            out.push((row, owner));
        }
    }
    out
}

/// Public helpers used by tests to exercise hit-testing without a live terminal.
#[must_use]
pub fn hit_test_transcript_for_test(app: &TuiApp, column: u16, row: u16) -> Option<TranscriptHit> {
    hit_test_transcript(app, column, row)
}

/// Toggle helpers used by the reducer for index-targeted clicks.
pub fn toggle_reasoning_at(transcript: &mut TranscriptState, index: usize) -> bool {
    let Some(item) = transcript.items.get_mut(index) else {
        return false;
    };
    if item.kind != TranscriptItemKind::Reasoning {
        return false;
    }
    item.toggle_reasoning();
    true
}

pub fn toggle_tool_output_at(transcript: &mut TranscriptState, index: usize) -> bool {
    let Some(item) = transcript.items.get_mut(index) else {
        return false;
    };
    item.toggle_tool_output()
}
