use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::{Frame, Terminal};

use crate::app::format_token_count;
use crate::{
    ApprovalOption, ConnectionState, LocalDiagnosticSeverity, Palette, Semantic,
    SlashSuggestionState, TerminalTheme, TranscriptItem, TranscriptItemKind, TuiApp,
};

pub fn render_app(frame: &mut Frame<'_>, app: &TuiApp) {
    render_chat(frame, app);
}

pub fn render_app_to_string(app: &TuiApp, width: u16, height: u16) -> std::io::Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render_app(frame, app))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

fn render_chat(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let approval_visible = app.approval_modal_visible();
    // Approval owns the bottom chrome (Claude Code style): it replaces the
    // composer and slash suggestions so the decision surface sits where input
    // normally is, instead of a centered floating modal.
    let suggestion_height = if approval_visible {
        0
    } else {
        slash_suggestion_height(&app.composer.slash_suggestions)
    };
    // Always keep ≥1 blank row between the transcript and the input chrome so
    // messages never butt against the composer. Active turns use 2 rows (blank +
    // activity line). Hide this gap while the approval surface owns the bottom.
    let gap_height = pre_input_gap_height(approval_visible, app.turn_state().is_active());

    // Bottom input slot: either the approval panel (auto-fit content height) or
    // the composer (grows with typed lines). While approval is up it covers the
    // status footer (model/cwd/etc.) so the decision surface owns the bottom.
    let footer_height: u16 = if approval_visible { 0 } else { 1 };
    let input_height = if approval_visible {
        approval_panel_height_for(app, area.height)
    } else {
        // Composer height grows with content: the two border rows plus one row
        // per logical line, capped so a long paste can't swallow the transcript.
        // Wrapping is off, so one logical line = one screen row.
        composer_height_for(app)
    };

    // Bottom-anchored stack: transcript (flex) → gap (blank / blank+activity) →
    // [slash suggestions] → composer|approval → [status footer]. Keeping
    // suggestions next to the composer means they never interrupt
    // transcript/activity spacing. Approval may claim almost the full height
    // (Claude Code style), so the transcript min shrinks to 1 row while it is up.
    let mut constraints = vec![Constraint::Min(if approval_visible { 1 } else { 3 })];
    if gap_height > 0 {
        constraints.push(Constraint::Length(gap_height));
    }
    if suggestion_height > 0 {
        constraints.push(Constraint::Length(suggestion_height));
    }
    constraints.push(Constraint::Length(input_height)); // composer or approval
    if footer_height > 0 {
        constraints.push(Constraint::Length(footer_height)); // status footer
    }
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Walk the split rects in the same order the constraints were pushed.
    let primary_area = vertical[0];
    let mut idx = 1;
    let gap_area = if gap_height > 0 {
        let rect = vertical[idx];
        idx += 1;
        Some(rect)
    } else {
        None
    };
    let suggestion_area = if suggestion_height > 0 {
        let rect = vertical[idx];
        idx += 1;
        Some(rect)
    } else {
        None
    };
    let input_area = vertical[idx];
    let status_area = if footer_height > 0 {
        Some(vertical[idx + 1])
    } else {
        None
    };

    // Side panels (status/trace/tasks/extensions/diff/command) are intentionally
    // not drawn: the product shell is a single full-width transcript like Claude
    // Code. Status lives in the footer; model/session/theme use centered modals.
    // Approval is bottom-docked chrome (not a modal) and hides the footer.
    render_primary(frame, app, primary_area);

    if let Some(gap_area) = gap_area {
        // Idle: the single reserved row stays blank. Active: blank row on top,
        // live activity prompt on the second row.
        if app.turn_state().is_active() && gap_area.height >= 2 {
            let prompt_area = Rect::new(
                gap_area.x,
                gap_area.y.saturating_add(1),
                gap_area.width,
                gap_area.height.saturating_sub(1),
            );
            render_activity_line(frame, app, prompt_area);
        }
    }
    if let Some(suggestion_area) = suggestion_area {
        render_slash_suggestions(
            frame,
            &app.composer.slash_suggestions,
            app.theme,
            suggestion_area,
        );
    }
    if approval_visible {
        render_approval_panel(frame, app, input_area);
    } else {
        render_composer(frame, app, input_area);
    }
    if let Some(status_area) = status_area {
        render_status(frame, app, status_area);
    }

    // Transient toast sits on the last row of the pre-composer gap (directly
    // above the input) when that gap exists; otherwise falls back to the last
    // row of the primary area. Auto-expires; never a red chip.
    if app.toast.is_some() {
        let toast_host = gap_area.unwrap_or(primary_area);
        render_toast(frame, app, toast_host);
    }

    // The model picker is a centered overlay, drawn last so it sits above the
    // chat. It never coexists with the approval surface (opening one is a
    // deliberate, separate user action), but the approval gate wins if both are
    // somehow active so a pending safety decision is never hidden.
    if app.model_picker.is_open() && !approval_visible {
        render_model_modal(frame, app, area);
    }

    // The session picker is the same centered-overlay shape. The approval gate
    // still wins if both are somehow active, and the model picker takes priority
    // over it (they are mutually exclusive in practice, but this keeps a single
    // deterministic winner).
    if app.session_panel.is_open() && !approval_visible && !app.model_picker.is_open() {
        render_session_modal(frame, app, area);
    }

    // Theme picker: same centered-overlay shape. Approval still wins; model and
    // session take priority so a deliberate switch is never buried under theme.
    if app.theme_picker.is_open()
        && !approval_visible
        && !app.model_picker.is_open()
        && !app.session_panel.is_open()
    {
        render_theme_modal(frame, app, area);
    }

    // Status popup: same centered-overlay shape, read-only. Every other modal
    // takes priority so a pending decision or deliberate switch is never buried.
    if app.status_panel.is_open()
        && !approval_visible
        && !app.model_picker.is_open()
        && !app.session_panel.is_open()
        && !app.theme_picker.is_open()
    {
        render_status_modal(frame, app, area);
    }
}

fn render_primary(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    if app.transcript.items.is_empty() {
        render_welcome(frame, app, area);
    } else {
        render_transcript(frame, app, area);
    }
}

fn render_transcript(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let transcript = &app.transcript;
    let theme = app.theme;
    let gutter = transcript_horizontal_gutter(area.width);
    let content_area = area.inner(Margin {
        horizontal: gutter,
        vertical: 0,
    });
    // Streaming labels share the turn spinner frame so their marquee stays in
    // lockstep with the activity line. Completed items ignore the frame.
    let lines = transcript_styled_lines(
        &transcript.items,
        theme,
        app.spinner_frame,
        app.markdown_enabled,
        usize::from(content_area.width),
        app.hovered_transcript_item(),
    );
    let wrapped = wrap_transcript_lines(lines, content_area.width);
    // After wrapping, pad every line that carries a block background (user/model
    // message blocks) out to the content width. The outer gutter keeps messages
    // away from the terminal edges while each message still reads as one block.
    let padded = pad_block_backgrounds(wrapped, content_area.width);
    let visible_start = transcript_visible_start(
        padded.len(),
        usize::from(content_area.height),
        transcript.scroll_offset,
    );
    let visible = transcript_visible_lines(&padded, content_area.height, transcript.scroll_offset);
    if visible.is_empty() || content_area.width == 0 || content_area.height == 0 {
        return;
    }

    // Messages pin to the top of the transcript region (chat-app default). When the
    // content is shorter than the viewport it starts at y=0 with empty space below;
    // when it overflows, scroll_offset=0 keeps the latest lines in view and the user
    // scrolls up to read earlier history. The activity prompt (if any) still sits in
    // its own reserved rows under this area.
    //
    // If an in-app selection is active, paint selected cells with a reverse
    // highlight so drag-select remains visible under mouse capture.
    if let Some(range) = selection_paint_range(app, content_area, visible_start) {
        let highlighted = apply_selection_highlight(visible, range, content_area.width);
        frame.render_widget(Paragraph::new(highlighted), content_area);
    } else {
        frame.render_widget(Paragraph::new(visible), content_area);
    }
}

/// Active selection range in content-local coordinates, if any. Only paints when
/// the live/committed gesture targets the **transcript** — a composer drag uses
/// a separate highlight path so the two surfaces never cross-paint.
fn selection_paint_range(
    app: &TuiApp,
    content_area: Rect,
    visible_start: usize,
) -> Option<crate::SelectionRange> {
    if app.selection.region != crate::SelectionRegion::Transcript {
        return None;
    }
    let geo = crate::ContentGeometry {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: content_area.height,
    };
    app.selection.paint_transcript_range(&geo, visible_start)
}

/// Reverse-video the cells inside `range` so the selection is visible. Operates
/// on already-wrapped visible lines (one Line per screen row).
fn apply_selection_highlight(
    lines: Vec<Line<'static>>,
    range: crate::SelectionRange,
    width: u16,
) -> Vec<Line<'static>> {
    let n = range.normalized();
    lines
        .into_iter()
        .enumerate()
        .map(|(row_idx, line)| {
            let row = row_idx as u16;
            if row < n.start_row || row > n.end_row {
                return line;
            }
            // Flatten spans to **display cells** (wide glyphs occupy 2 cells) so
            // selection highlighting matches mouse/cursor columns for CJK.
            let mut cells: Vec<(char, Style, u16)> = Vec::new(); // ch, style, display_width
            for span in &line.spans {
                let style = span.style;
                for ch in span.content.chars() {
                    let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
                    let w = w.max(1);
                    cells.push((ch, style, w));
                }
            }
            // Pad to content width in display columns.
            let mut used: u16 = cells.iter().map(|c| c.2).sum();
            while used < width {
                cells.push((' ', line.style, 1));
                used = used.saturating_add(1);
            }
            let mut display_col: u16 = 0;
            let mut spans = Vec::new();
            let mut buf = String::new();
            let mut cur_style = Style::default();
            let mut started = false;
            for (ch, style, w) in cells {
                let mut style = style;
                // Mark glyph selected if any of its display columns fall in range.
                let mut selected = false;
                for offset in 0..w {
                    if n.contains_cell(display_col.saturating_add(offset), row) {
                        selected = true;
                        break;
                    }
                }
                if selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                display_col = display_col.saturating_add(w);
                if !started {
                    cur_style = style;
                    buf.push(ch);
                    started = true;
                } else if style == cur_style {
                    buf.push(ch);
                } else {
                    spans.push(Span::styled(std::mem::take(&mut buf), cur_style));
                    cur_style = style;
                    buf.push(ch);
                }
            }
            if !buf.is_empty() {
                spans.push(Span::styled(buf, cur_style));
            }
            Line::from(spans).style(line.style)
        })
        .collect()
}

fn transcript_horizontal_gutter(width: u16) -> u16 {
    u16::from(width >= 4)
}

/// Pad any line that carries a background fill (its `Line::style` has a `bg`) out
/// to the transcript content width with trailing spaces in that same style, so
/// message-block backgrounds render as a full strip inside the horizontal gutter.
/// Lines with no background are returned unchanged. Empty separator rows are never
/// padded (they have no `bg`), so item spacing stays a single plain blank line.
fn pad_block_backgrounds(lines: Vec<Line<'static>>, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    lines
        .into_iter()
        .map(|mut line| {
            if line.style.bg.is_none() {
                return line;
            }
            let used = line.width();
            if used < width {
                line.spans
                    .push(Span::styled(" ".repeat(width - used), line.style));
            }
            line
        })
        .collect()
}

fn render_welcome(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let gutter = transcript_horizontal_gutter(area.width);
    // Keep vertical margin only when the region is tall enough; short terminals
    // need every row for logo + facts.
    let v_margin = u16::from(area.height >= 12);
    let content = area.inner(Margin {
        horizontal: gutter.max(1),
        vertical: v_margin,
    });
    if content.width == 0 || content.height == 0 {
        return;
    }
    let lines = crate::welcome::welcome_lines_for(
        app,
        usize::from(content.width),
        usize::from(content.height),
        app.theme,
    );
    // Only pad when there is spare vertical room so short terminals still show
    // Model / Workspace / Extensions under the logo.
    let needed = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let spare = content.height.saturating_sub(needed);
    let y_pad = if spare > 2 { spare / 4 } else { 0 };
    let block = Rect {
        x: content.x,
        y: content.y.saturating_add(y_pad),
        width: content.width,
        height: content.height.saturating_sub(y_pad),
    };
    frame.render_widget(Paragraph::new(lines), block);
}

pub(crate) fn transcript_visible_lines(
    lines: &[Line<'static>],
    area_height: u16,
    scroll_offset: u16,
) -> Vec<Line<'static>> {
    if lines.is_empty() {
        return Vec::new();
    }

    let visible_height = usize::from(area_height.max(1));
    let start = transcript_visible_start(lines.len(), visible_height, scroll_offset);
    let end = (start + visible_height).min(lines.len());
    lines[start..end].to_vec()
}

pub(crate) fn wrap_transcript_lines(
    lines: Vec<Line<'static>>,
    area_width: u16,
) -> Vec<Line<'static>> {
    let width = usize::from(area_width.max(1));
    lines
        .into_iter()
        .flat_map(|line| wrap_line(line, width))
        .collect()
}

fn wrap_line(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    if line.width() <= width {
        return vec![line];
    }

    let style = line.style;
    let alignment = line.alignment;
    let mut output = Vec::new();
    let mut current = Vec::new();
    let mut current_width = 0usize;

    for span in line.spans {
        for character in span.content.chars() {
            let text = character.to_string();
            let character_width = Span::raw(text.as_str()).width();
            if current_width > 0 && current_width + character_width > width {
                output.push(line_from_spans(
                    std::mem::take(&mut current),
                    style,
                    alignment,
                ));
                current_width = 0;
            }
            current.push(Span::styled(text, span.style));
            current_width += character_width;
        }
    }

    if current.is_empty() {
        output.push(line_from_spans(Vec::new(), style, alignment));
    } else {
        output.push(line_from_spans(current, style, alignment));
    }

    output
}

fn line_from_spans(
    spans: Vec<Span<'static>>,
    style: Style,
    alignment: Option<Alignment>,
) -> Line<'static> {
    Line {
        style,
        alignment,
        spans,
    }
}

fn transcript_styled_lines(
    items: &[TranscriptItem],
    theme: TerminalTheme,
    marquee_frame: usize,
    markdown_enabled: bool,
    content_width: usize,
    hovered_item: Option<usize>,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut rendered_any = false;
    for (index, item) in items.iter().enumerate() {
        let hovered = hovered_item == Some(index);
        let item_lines = transcript_item_lines(
            item,
            theme,
            marquee_frame,
            markdown_enabled,
            content_width,
            hovered,
        );
        // Skip ghost items that render no rows (empty model deltas, whitespace-only
        // bodies, etc.). Including them would inject bare separators and look like
        // extra blank rows between real messages.
        if item_lines.is_empty() {
            continue;
        }
        if rendered_any {
            // Exactly one plain blank row between every adjacent *visible* item.
            // User blocks also keep their own bottom background padding strip, so
            // user → thinking reads as: [user text] / [user bg pad] / [plain blank]
            // / [thinking] — two spacer rows, one of which carries the user bubble.
            lines.push(Line::from(""));
        }
        rendered_any = true;
        lines.extend(item_lines);
    }
    // Final safety net: collapse any run of visually blank rows (no background and
    // no visible text) down to a single blank, so ghost content can never stack
    // separators into two/three empty rows. Background-filled user padding is not
    // collapsed, so user → next item still keeps pad + separator.
    collapse_visually_blank_runs(lines)
}

/// Drop consecutive plain blank rows, keeping at most one between content.
/// Rows with a background fill (user message padding) are never collapsed — they
/// are part of the message block, not inter-item spacing.
fn collapse_visually_blank_runs(lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut previous_blank = false;
    for line in lines {
        let is_blank = line.style.bg.is_none()
            && line
                .spans
                .iter()
                .all(|span| span.content.chars().all(char::is_whitespace));
        if is_blank {
            if previous_blank {
                continue;
            }
            previous_blank = true;
            // Normalize to a plain empty line so residual whitespace never paints.
            out.push(Line::from(""));
            continue;
        }
        previous_blank = false;
        out.push(line);
    }
    out
}

/// Hit-test twin of [`collapse_visually_blank_runs`]: collapses plain blanks
/// while preserving the `Option<item_index>` owner of every kept row.
pub(crate) fn collapse_visually_blank_runs_for_hit(
    lines: Vec<(Line<'static>, Option<usize>)>,
) -> Vec<(Line<'static>, Option<usize>)> {
    let mut out = Vec::with_capacity(lines.len());
    let mut previous_blank = false;
    for (line, owner) in lines {
        let is_blank = line.style.bg.is_none()
            && line
                .spans
                .iter()
                .all(|span| span.content.chars().all(char::is_whitespace));
        if is_blank {
            if previous_blank {
                continue;
            }
            previous_blank = true;
            out.push((Line::from(""), None));
            continue;
        }
        previous_blank = false;
        out.push((line, owner));
    }
    out
}

/// Hit-test twin of [`pad_block_backgrounds`].
pub(crate) fn pad_block_backgrounds_for_hit(
    lines: Vec<(Line<'static>, Option<usize>)>,
    width: u16,
) -> Vec<(Line<'static>, Option<usize>)> {
    let width = usize::from(width);
    lines
        .into_iter()
        .map(|(mut line, owner)| {
            if line.style.bg.is_none() {
                return (line, owner);
            }
            let used = line.width();
            if used < width {
                line.spans
                    .push(Span::styled(" ".repeat(width - used), line.style));
            }
            (line, owner)
        })
        .collect()
}

/// First index into the full line list that the visible window starts at, given
/// a bottom-anchored scroll_offset. Shared by render and hit-testing.
pub(crate) fn transcript_visible_start(
    total_lines: usize,
    visible_height: usize,
    scroll_offset: u16,
) -> usize {
    if total_lines == 0 || visible_height == 0 {
        return 0;
    }
    let max_offset = total_lines.saturating_sub(visible_height);
    let offset = usize::from(scroll_offset).min(max_offset);
    total_lines.saturating_sub(visible_height + offset)
}

/// Hit-test twin of [`transcript_item_lines`]. Marquee is frozen at frame 0 so
/// hit geometry is stable (the marquee only recolors characters, never changes
/// their count or width). Hover is off so color-only hover does not affect
/// geometry either.
pub(crate) fn transcript_item_lines_for_hit(
    item: &TranscriptItem,
    theme: TerminalTheme,
    markdown_enabled: bool,
    content_width: usize,
) -> Vec<Line<'static>> {
    transcript_item_lines(item, theme, 0, markdown_enabled, content_width, false)
}

fn transcript_item_lines(
    item: &TranscriptItem,
    theme: TerminalTheme,
    marquee_frame: usize,
    markdown_enabled: bool,
    content_width: usize,
    hovered: bool,
) -> Vec<Line<'static>> {
    let style = transcript_item_style(item.kind, theme);
    let body = trim_blank_boundary_lines(&item.body);
    // Whitespace-only bodies become a single space from the trim helper. Treat
    // that as empty so we don't paint a ghost blank row for a placeholder item.
    let body_is_empty = body.trim().is_empty();
    match item.kind {
        TranscriptItemKind::User => {
            if body_is_empty {
                return Vec::new();
            }
            user_message_block_lines(&body, style)
        }
        TranscriptItemKind::Model => {
            if body_is_empty {
                // Empty / pending model streams must not reserve a blank row.
                return Vec::new();
            }
            if markdown_enabled {
                crate::markdown_view::markdown_to_lines(
                    &body,
                    style,
                    theme,
                    content_width,
                    item.streaming,
                )
            } else {
                message_block_lines(&body, "", style)
            }
        }
        TranscriptItemKind::Reasoning => reasoning_lines(
            item,
            &body,
            theme,
            marquee_frame,
            markdown_enabled,
            content_width,
            hovered,
        ),
        TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => {
            tool_call_lines(item, &body, item.kind, theme, marquee_frame, content_width)
        }
        TranscriptItemKind::Approval => {
            if body_is_empty {
                return Vec::new();
            }
            let mut spans =
                status_label_spans("approval", item.kind, false, false, theme, marquee_frame);
            spans.push(Span::raw(" "));
            spans.push(Span::styled(body, style));
            vec![Line::from(spans)]
        }
        TranscriptItemKind::Diagnostic => {
            if body_is_empty && item.title.trim().is_empty() {
                return Vec::new();
            }
            let mut spans =
                status_label_spans("diagnostic", item.kind, false, false, theme, marquee_frame);
            spans.push(Span::raw(" "));
            spans.push(Span::styled(item.title.clone(), style));
            vec![
                Line::from(spans),
                Line::from(Span::styled(format!("  {body}"), style)),
            ]
        }
        TranscriptItemKind::Progress => {
            if body_is_empty {
                return Vec::new();
            }
            let mut spans =
                status_label_spans("note", item.kind, false, false, theme, marquee_frame);
            spans.push(Span::raw(" "));
            spans.push(Span::styled(body, style));
            vec![Line::from(spans)]
        }
    }
}

/// Default number of result rows shown under a non-shell tool call. Shell output
/// stays fully collapsed until click/Ctrl+O expands it.
const TOOL_RESULT_DEFAULT_ROWS: usize = 2;

/// Render a tool call as tense-aware status + muted target:
///   ✹ Writing... src/main.rs   (live: star spinner prefix)
///   • Wrote src/main.rs        (done: bullet)
///   ✹ Reading... crates/foo.rs
///   • Read crates/foo.rs
///   ✹ Editing... path.rs
///       --- path.rs
///       +++ path.rs
///       -old
///       +new
///   ✹ Running... cargo test
///   • Ran cargo test
/// Body format is preferably `friendly_name target\nresult…` (legacy
/// `friendly_name target - result` is still accepted).
///
/// Tool labels never marquee the verb — progressive tense is the word cue
/// (`Writing...`); the star spinner prefix marks in-progress rows. Settled
/// past tense (`Wrote`) uses a solid bullet.
fn tool_call_lines(
    item: &TranscriptItem,
    body: &str,
    kind: TranscriptItemKind,
    theme: TerminalTheme,
    marquee_frame: usize,
    content_width: usize,
) -> Vec<Line<'static>> {
    let (head, detail) = match body.split_once('\n') {
        // Preserve source whitespace inside diff/output rows. Trimming all
        // whitespace here erased leading tabs and trailing spaces from the first
        // or last changed line before the styled diff renderer saw them.
        Some((head, detail)) => (
            head.trim(),
            detail.trim_matches(|character| character == '\r' || character == '\n'),
        ),
        None => match body.split_once(" - ") {
            Some((head, detail)) => (head.trim(), detail.trim()),
            None => (body.trim(), ""),
        },
    };

    // Split "friendly_name target…" so the name can carry the status bullet.
    let (tool_label, target) = match head.split_once(' ') {
        Some((name, rest)) => (name, rest.trim()),
        None => (head, ""),
    };
    // Tool phase machine rendered as tense words:
    //   ToolCall + !streaming → Waiting...
    //   ToolCall + streaming  → Running... / Writing...
    //   ToolResult            → Ran / Wrote
    let phase = tool_phase(kind, item.streaming);
    let is_shell = tool_label == "shell";
    let is_edit = tool_label == "edit_file" || tool_label == "write_file";
    let status_word = tool_tense_label(tool_label, phase);
    let meta = transcript_meta_style(theme);

    // Live tools: spinning star prefix + solid verb. Done: bullet + past tense.
    let live = phase != ToolPhase::Done;
    let mut head_spans = status_label_spans(
        &status_word,
        TranscriptItemKind::ToolCall,
        live,
        false, // no letter marquee on tool verbs
        theme,
        marquee_frame,
    );
    if !target.is_empty() {
        head_spans.push(Span::raw(" "));
        if is_shell {
            // Shell: token-level syntax highlight (ls orange, -a cyan, …) + italic.
            head_spans.extend(crate::shell_highlight::highlight_shell_command(
                target, theme,
            ));
        } else {
            // Paths stay muted + italic (no underline).
            head_spans.push(Span::styled(target.to_string(), tool_path_style(theme)));
        }
    }
    let mut lines = vec![Line::from(head_spans)];

    // Waiting / Running: never paint raw observation junk under the header.
    // Results only appear once the tool is Done.
    if phase != ToolPhase::Done || detail.is_empty() {
        return lines;
    }

    let detail_rows: Vec<&str> = detail
        .lines()
        .filter(|row| !row.trim().is_empty())
        .collect();
    if detail_rows.is_empty() {
        return lines;
    }

    let is_diff = detail_looks_like_diff(detail);
    // File edits with a diff body expand by default so the change is visible
    // without an extra click. Finished shell shows its output (auto-expanded
    // by the transcript reducer). Other tools preview a couple of lines.
    let expanded =
        item.tool_output_expanded || (is_edit && is_diff) || (is_shell && phase == ToolPhase::Done);

    // Edit/write diffs get a VSCode-like colored renderer (side-by-side when
    // the terminal is wide enough, unified when narrow). Other tools keep the
    // plain connector + mono rows.
    if is_edit && is_diff && expanded {
        lines.extend(crate::diff_view::render_edit_diff_lines(
            detail,
            content_width,
            theme,
        ));
        return lines;
    }

    let default_rows = if is_shell {
        if phase == ToolPhase::Done {
            detail_rows.len()
        } else {
            0
        }
    } else if is_edit && is_diff {
        detail_rows.len()
    } else {
        TOOL_RESULT_DEFAULT_ROWS
    };
    let shown = if expanded {
        detail_rows.len()
    } else {
        detail_rows.len().min(default_rows)
    };
    for (index, row) in detail_rows.iter().take(shown).enumerate() {
        let connector = if index == 0 { "  ⎿ " } else { "    " };
        let row_style = if is_diff {
            diff_line_style(row, theme)
        } else {
            meta
        };
        lines.push(Line::from(vec![
            Span::styled(connector, meta),
            Span::styled((*row).to_string(), row_style),
        ]));
    }
    if !expanded && !is_shell && detail_rows.len() > shown {
        let hidden = detail_rows.len() - shown;
        lines.push(Line::from(Span::styled(
            format!("    … +{hidden} lines (click / ctrl+o to expand)"),
            meta,
        )));
    }
    lines
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ToolPhase {
    /// Proposed / awaiting approval — no execution yet.
    Waiting,
    /// Actively executing after permission (or auto-allow).
    Running,
    /// Finished — show past tense + user-facing result.
    Done,
}

fn tool_phase(kind: TranscriptItemKind, streaming: bool) -> ToolPhase {
    match kind {
        TranscriptItemKind::ToolResult => ToolPhase::Done,
        TranscriptItemKind::ToolCall if streaming => ToolPhase::Running,
        TranscriptItemKind::ToolCall => ToolPhase::Waiting,
        _ => ToolPhase::Done,
    }
}

/// Waiting / progressive / past-tense verb for a friendly tool label.
fn tool_tense_label(tool_label: &str, phase: ToolPhase) -> String {
    match (tool_label, phase) {
        ("write_file", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("write_file", ToolPhase::Running) => "Writing...".to_string(),
        ("write_file", ToolPhase::Done) => "Wrote".to_string(),

        ("read_file", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("read_file", ToolPhase::Running) => "Reading...".to_string(),
        ("read_file", ToolPhase::Done) => "Read".to_string(),

        ("edit_file", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("edit_file", ToolPhase::Running) => "Editing...".to_string(),
        ("edit_file", ToolPhase::Done) => "Edited".to_string(),

        ("search", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("search", ToolPhase::Running) => "Searching...".to_string(),
        ("search", ToolPhase::Done) => "Searched".to_string(),

        ("shell", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("shell", ToolPhase::Running) => "Running...".to_string(),
        ("shell", ToolPhase::Done) => "Ran".to_string(),

        ("git_status", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("git_status", ToolPhase::Running) => "Checking...".to_string(),
        ("git_status", ToolPhase::Done) => "Checked".to_string(),

        ("git_diff", ToolPhase::Waiting) => "Waiting...".to_string(),
        ("git_diff", ToolPhase::Running) => "Diffing...".to_string(),
        ("git_diff", ToolPhase::Done) => "Diffed".to_string(),

        (_, ToolPhase::Waiting) => "Waiting...".to_string(),
        (_, ToolPhase::Running) => "Running...".to_string(),
        (other, ToolPhase::Done) if other.is_empty() => "Ran".to_string(),
        (other, ToolPhase::Done) => other.to_string(),
    }
}

/// Path targets next to Read/Wrote/Edited: muted meta + italic (no underline).
fn tool_path_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).meta()
}

fn detail_looks_like_diff(detail: &str) -> bool {
    let mut saw_hunk = false;
    let mut saw_change = false;
    for line in detail.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") || line.starts_with("@@") {
            saw_hunk = true;
        } else if (line.starts_with('+') && !line.starts_with("+++"))
            || (line.starts_with('-') && !line.starts_with("---"))
        {
            saw_change = true;
        }
        if saw_hunk && saw_change {
            return true;
        }
    }
    // Accept pure +/- dumps without headers (some tools emit only that).
    saw_change && detail.lines().count() > 1
}

fn diff_line_style(line: &str, theme: TerminalTheme) -> Style {
    let palette = Palette::for_theme(theme);
    if line.starts_with('+') && !line.starts_with("+++") {
        return palette.diff_added();
    }
    if line.starts_with('-') && !line.starts_with("---") {
        return palette.diff_removed();
    }
    if line.starts_with("@@") {
        return palette.meta().add_modifier(Modifier::BOLD);
    }
    // File headers (`--- a`, `+++ b`) stay muted meta.
    palette.meta()
}

/// Remove blank boundary rows contributed by provider/event payloads while
/// preserving intentional blank rows and indentation inside the message body.
/// Collapses any run of interior blank rows to a single empty line so item
/// spacing never competes with body-internal gaps.
fn trim_blank_boundary_lines(body: &str) -> String {
    let rows: Vec<&str> = body.lines().collect();
    let Some(start) = rows.iter().position(|row| !row.trim().is_empty()) else {
        return " ".to_string();
    };
    let end = rows
        .iter()
        .rposition(|row| !row.trim().is_empty())
        .unwrap_or(start);
    let mut out = Vec::new();
    let mut previous_blank = false;
    for row in &rows[start..=end] {
        let blank = row.trim().is_empty();
        if blank {
            if previous_blank {
                continue;
            }
            out.push("");
            previous_blank = true;
        } else {
            out.push(*row);
            previous_blank = false;
        }
    }
    out.join("\n")
}

/// Render a compact user/model message block. The transcript owns one plain
/// separator row between adjacent items. `marker` prefixes the first text row,
/// while continuation rows align with that marker.
fn message_block_lines(body: &str, marker: &str, style: Style) -> Vec<Line<'static>> {
    let continuation_prefix = if marker.is_empty() { "" } else { "  " };
    body.lines()
        .enumerate()
        .map(|(index, line)| {
            let prefix = if index == 0 {
                marker
            } else {
                continuation_prefix
            };
            Line::from(Span::styled(format!("{prefix}{line}"), style)).style(style)
        })
        .collect()
}

/// User messages get a padded block: one blank background row above the text,
/// the text row(s) with the `❯ ` marker, and one blank background row below.
/// The bottom padding is part of the bubble; `transcript_styled_lines` still adds
/// one plain inter-item blank after the user block, so the gap before thinking /
/// model / tools is pad + separator (two rows total, one with user bg).
fn user_message_block_lines(body: &str, style: Style) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    // Top padding strip (background only).
    lines.push(Line::from(Span::styled(" ", style)).style(style));
    for (index, line) in body.lines().enumerate() {
        let prefix = if index == 0 { "❯ " } else { "  " };
        lines.push(Line::from(Span::styled(format!("{prefix}{line}"), style)).style(style));
    }
    // Bottom padding strip (background only).
    lines.push(Line::from(Span::styled(" ", style)).style(style));
    lines
}

/// Render a reasoning ("thinking") item as a collapsible block:
///   ✹ Thinking... [3s · 100 tokens] (click or ctrl+p to expand)
///   • Thought [3s · 100 tokens] (click or ctrl+p to collapse)
/// The header is muted by default and brightens on hover as a click affordance.
/// Duration and tokens update live while streaming; duration freezes on finish.
/// Empty settled Thoughts (no summary) are suppressed — they would only show as
/// the useless `[0s · 0 tokens]` ghost row.
fn reasoning_lines(
    item: &TranscriptItem,
    body: &str,
    theme: TerminalTheme,
    marquee_frame: usize,
    markdown_enabled: bool,
    content_width: usize,
    hovered: bool,
) -> Vec<Line<'static>> {
    let collapsed = item.is_reasoning_collapsed();
    let palette = Palette::for_theme(theme);
    // Only the expanded thinking *body* is italic. The Thinking/Thought label
    // stays upright + bold; body drops bold so the aside does not shout.
    let body_style = transcript_item_style(TranscriptItemKind::Reasoning, theme)
        .remove_modifier(Modifier::BOLD)
        .add_modifier(Modifier::ITALIC);
    // Stats / expand-hint chrome must stay softer than the status word and only
    // brighten on hover. Avoid Semantic::Meta (terminal Gray often reads as
    // primary on modern themes).
    let detail_muted = reasoning_detail_style(theme);

    // The reasoning body, minus the pending placeholder sentinel (which is just a
    // "waiting for content" marker, not real reasoning).
    let has_content = body != "thinking" && !body.trim().is_empty();

    // Settled empty Thought rows are noise (no summary, no tokens). Prefer not to
    // paint them even if one slips past transcript cleanup.
    if !item.streaming && !has_content {
        return Vec::new();
    }

    // Prefer the live counter on the item; re-estimate from body so synthetic
    // test items that only set `body` still show a sensible token count.
    let tokens = if item.thinking_tokens > 0 {
        item.thinking_tokens
    } else {
        crate::transcript::estimate_thinking_tokens(body)
    };
    let seconds = item.thinking_seconds;
    let hint = if collapsed {
        "click or ctrl+p to expand"
    } else {
        "click or ctrl+p to collapse"
    };
    // Streaming headers always paint a fixed stats slot so the row width does
    // not jump the first time content/tokens/seconds appear (layout flash).
    // Settled empty Thoughts still suppress the whole row above.
    let detail = if item.streaming {
        format!("[{seconds}s · {tokens} tokens]")
    } else if has_content || tokens > 0 || seconds > 0 {
        format!("[{seconds}s · {tokens} tokens] ({hint})")
    } else {
        String::new()
    };

    // Tense: streaming → "Thinking..."; finished → "Thought". Always muted by
    // default; hover lifts the whole header to primary text. Progressive tense
    // uses ASCII "..." (not the unicode ellipsis). Live Thinking keeps the
    // two-tone marquee; settled Thought is solid (no marquee flash after done).
    let label = if item.streaming {
        "Thinking..."
    } else {
        "Thought"
    };
    let header_spans = if hovered {
        // Hover: solid primary color (no letter marquee) so the row reads as
        // clickable. Live rows still keep the spinning star prefix.
        let hover = palette.hover_emphasis();
        let marker = status_label_marker(item.streaming, marquee_frame);
        let mut spans = vec![
            Span::styled(format!("{marker} "), hover),
            Span::styled(label.to_string(), hover),
        ];
        if !detail.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(detail, hover));
        }
        spans
    } else {
        let mut spans = status_label_spans(
            label,
            TranscriptItemKind::Reasoning,
            item.streaming, // spinner prefix while Thinking...
            item.streaming, // letter marquee while Thinking...
            theme,
            marquee_frame,
        );
        if !detail.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(detail, detail_muted));
        }
        spans
    };
    let mut output = vec![Line::from(header_spans)];

    if collapsed || !has_content {
        return output;
    }

    // Expanded: show the reasoning content, indented as an aside (muted + italic).
    if markdown_enabled {
        // Leave room for the two-space aside indent applied by the helper.
        let md_width = content_width.saturating_sub(2);
        output.extend(crate::markdown_view::reasoning_markdown_lines(
            body,
            theme,
            md_width,
            item.streaming,
        ));
    } else {
        for line in body.lines() {
            output.push(Line::from(Span::styled(format!("  {line}"), body_style)));
        }
    }
    if item.streaming {
        // Streaming ellipsis is chrome, not body — keep it upright with the stats.
        output.push(Line::from(Span::styled("  ...", detail_muted)));
    }
    output
}

fn transcript_item_style(kind: TranscriptItemKind, theme: TerminalTheme) -> Style {
    let palette = Palette::for_theme(theme);
    match kind {
        TranscriptItemKind::User => palette.semantic(Semantic::User),
        TranscriptItemKind::Model => palette.semantic(Semantic::Model),
        TranscriptItemKind::Reasoning => palette.semantic(Semantic::Reasoning),
        TranscriptItemKind::Diagnostic => palette.semantic(Semantic::Error),
        TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => {
            palette.semantic(Semantic::Tool)
        }
        TranscriptItemKind::Approval => palette.semantic(Semantic::Approval),
        TranscriptItemKind::Progress => palette.meta(),
    }
}

/// Foreground style for a status-label marker + word. No background fill — labels
/// read as `• word` / `✹ word` with a status color instead of inverted chips.
fn status_label_style(kind: TranscriptItemKind, _live: bool, theme: TerminalTheme) -> Style {
    let palette = Palette::for_theme(theme);
    match kind {
        TranscriptItemKind::Diagnostic => palette.semantic(Semantic::Error),
        TranscriptItemKind::Approval => palette.semantic(Semantic::Approval),
        // Thinking / Thought / tools / shell stay muted — no blue or green accent.
        TranscriptItemKind::Reasoning => palette.semantic(Semantic::Reasoning),
        TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => {
            palette.semantic(Semantic::Tool)
        }
        TranscriptItemKind::Progress => palette.meta(),
        _ => palette.meta(),
    }
}

/// Marker glyph for a status label: spinning star while live, solid bullet when done.
fn status_label_marker(live: bool, frame: usize) -> &'static str {
    if live {
        WAITING_SPINNER_FRAMES[frame % WAITING_SPINNER_FRAMES.len()]
    } else {
        "•"
    }
}

/// Build `marker label` spans for a transcript status word.
///
/// - `live`: use the star spinner prefix (same frames as the activity line).
/// - `marquee_text`: two-tone letter marquee on the verb (Thinking... only).
/// Done rows always get a solid `•` prefix and solid text.
fn status_label_spans(
    label: &str,
    kind: TranscriptItemKind,
    live: bool,
    marquee_text: bool,
    theme: TerminalTheme,
    marquee_frame: usize,
) -> Vec<Span<'static>> {
    let base = status_label_style(kind, live, theme);
    let marker = status_label_marker(live, marquee_frame);
    let mut spans = vec![Span::styled(format!("{marker} "), base)];
    if marquee_text {
        spans.extend(marquee_spans(label, base, marquee_frame));
    } else {
        spans.push(Span::styled(label.to_string(), base));
    }
    spans
}

/// How many characters of the marquee highlight window. Small on purpose — a
/// short bright band that walks across the word reads as "alive" without noise.
const MARQUEE_WINDOW: usize = 2;

/// Split `text` into character spans whose styles alternate between the base
/// (normal) color and a brighter highlight. The highlight window advances with
/// `frame` so successive redraws look like a marquee / walking light.
fn marquee_spans(text: &str, base: Style, frame: usize) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let bright = marquee_bright_style(base);
    let len = chars.len();
    // Period slightly longer than the word so the bright band walks off-end
    // briefly, which reads better than wrapping tightly on short words.
    let period = len.saturating_add(MARQUEE_WINDOW).max(1);
    let head = frame % period;

    let mut spans = Vec::with_capacity(len);
    for (index, ch) in chars.into_iter().enumerate() {
        // Distance of this character from the marquee head, wrapping via the
        // longer period so characters past the word stay dark for a beat.
        let in_window = (0..MARQUEE_WINDOW).any(|offset| {
            let pos = (head + period - offset) % period;
            pos == index
        });
        let style = if in_window { bright } else { base };
        spans.push(Span::styled(ch.to_string(), style));
    }
    spans
}

/// Brighten a style for the marquee highlight: prefer a lighter sibling of the
/// same hue, falling back to BOLD when the color is unknown.
fn marquee_bright_style(base: Style) -> Style {
    let bright_fg = match base.fg {
        Some(Color::Gray) | Some(Color::DarkGray) => Color::White,
        Some(Color::Cyan) => Color::Rgb(165, 243, 252),
        Some(Color::Blue) => Color::Rgb(147, 197, 253),
        Some(Color::Green) => Color::Rgb(134, 239, 172),
        Some(Color::Yellow) => Color::Rgb(253, 224, 71),
        Some(Color::Red) => Color::Rgb(252, 165, 165),
        Some(Color::Magenta) => Color::Rgb(240, 171, 252),
        Some(Color::Rgb(r, g, b)) => {
            // Lift toward white without clipping past 255.
            Color::Rgb(
                r.saturating_add((255 - r) / 2),
                g.saturating_add((255 - g) / 2),
                b.saturating_add((255 - b) / 2),
            )
        }
        Some(other) => other,
        None => Color::White,
    };
    base.fg(bright_fg).add_modifier(Modifier::BOLD)
}

/// Fixed gutter width for the composer prompt marker (`❯ `).
pub(crate) const COMPOSER_GUTTER: u16 = 2;

/// Maximum number of text rows the composer grows to before it stops expanding
/// (a long paste scrolls within the box instead of swallowing the transcript).
const COMPOSER_MAX_TEXT_ROWS: u16 = 6;

/// Dynamic composer height: the two border rows plus one row per logical line,
/// capped at `COMPOSER_MAX_TEXT_ROWS` text rows. Grows as the user types multi-line
/// input, shrinks back when it is cleared.
/// Rows reserved between the transcript and the input chrome (composer / slash
/// suggestions). Always ≥1 blank when idle so messages never collide with the
/// input box; active turns use 2 (blank + activity line). Approval owns the
/// bottom surface and hides this gap.
#[must_use]
pub(crate) fn pre_input_gap_height(approval_visible: bool, turn_active: bool) -> u16 {
    if approval_visible {
        0
    } else if turn_active {
        2
    } else {
        1
    }
}

fn composer_height_for(app: &TuiApp) -> u16 {
    let text_rows = u16::try_from(app.composer.line_count())
        .unwrap_or(COMPOSER_MAX_TEXT_ROWS)
        .clamp(1, COMPOSER_MAX_TEXT_ROWS);
    text_rows.saturating_add(2)
}

/// Terminal-absolute rect of the composer's editable text area — the region right
/// of the `❯ ` gutter, inside the top/bottom rules. Mirrors the layout math in
/// [`render_chat`] and [`render_composer`] so in-app selection can map a pointer
/// onto composer text without handing the widget to ratatui. Returns `None` when
/// the composer is not on screen (approval owns the bottom) or there is no room.
#[must_use]
pub(crate) fn composer_text_area_rect(app: &TuiApp) -> Option<Rect> {
    if app.approval_modal_visible() {
        return None;
    }
    let width = app.terminal_size.width;
    let height = app.terminal_size.height;
    if width == 0 || height == 0 {
        return None;
    }

    // Bottom-anchored stack (see `render_chat`): the composer sits directly above
    // the one-row status footer, so its top-left y is height − footer − input.
    let footer_height: u16 = 1;
    let input_height = composer_height_for(app);
    let input_y = height
        .saturating_sub(footer_height)
        .saturating_sub(input_height);

    // Within the input slot: horizontal gutter, then the top rule (1 row), then the
    // fixed `❯ ` prompt gutter before the text.
    let gutter = transcript_horizontal_gutter(width);
    let content_width = width.saturating_sub(gutter.saturating_mul(2));
    let text_x = gutter.saturating_add(COMPOSER_GUTTER);
    let text_width = content_width.saturating_sub(COMPOSER_GUTTER);
    let text_y = input_y.saturating_add(1);
    let text_height = input_height.saturating_sub(2);
    if text_width == 0 || text_height == 0 {
        return None;
    }
    Some(Rect::new(text_x, text_y, text_width, text_height))
}

fn render_composer(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let composer = &app.composer;
    let theme = app.theme;
    let content_area = area.inner(Margin {
        horizontal: transcript_horizontal_gutter(area.width),
        vertical: 0,
    });
    // The composer uses the same horizontal gutter as transcript messages. Its
    // top/bottom rules are intentionally light so they frame input without
    // competing with content.
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(composer_border_style(theme, app.status.reasoning_effort));
    let inner = block.inner(content_area);
    frame.render_widget(block, content_area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // A fixed `❯ ` gutter on the first row; the text area fills the rest. The
    // gutter is not repeated per wrapped row (it marks the prompt, not each line).
    // Color matches the top/bottom composer rules so the marker reads as chrome.
    frame.render_widget(
        Paragraph::new("❯").style(composer_border_style(theme, app.status.reasoning_effort)),
        Rect::new(inner.x, inner.y, COMPOSER_GUTTER.min(inner.width), 1),
    );

    let text_area = Rect::new(
        inner.x.saturating_add(COMPOSER_GUTTER),
        inner.y,
        inner.width.saturating_sub(COMPOSER_GUTTER),
        inner.height,
    );
    if text_area.width == 0 {
        return;
    }

    // We render the composer text ourselves rather than handing the `TextArea`
    // widget to ratatui. The widget maintains its own horizontal-scroll viewport
    // and draws its own cursor cell; rendering it inside our offset gutter fought
    // the terminal cursor we set below and made freshly typed characters appear to
    // land *before* earlier ones. `TextArea` stays the edit model (cursor motion,
    // word ops, multi-line); the view is a plain left-to-right Paragraph of its
    // logical lines, which cannot reorder.
    let lines = composer.lines();
    if lines.iter().all(|line| line.is_empty()) {
        // Empty composer: show the dim placeholder. Selection never paints over
        // the placeholder (there's nothing useful to copy).
        frame.render_widget(
            Paragraph::new(crate::COMPOSER_PLACEHOLDER)
                .style(Palette::for_theme(theme).placeholder()),
            text_area,
        );
    } else {
        let text: Vec<Line<'static>> = lines
            .iter()
            .map(|line| Line::from(Span::raw(line.clone())))
            .collect();
        // In-app selection for the composer uses the same reverse-video path as
        // the transcript, but only when the gesture started in this text area.
        if app.selection.region == crate::SelectionRegion::Composer {
            let geo = crate::ContentGeometry {
                x: text_area.x,
                y: text_area.y,
                width: text_area.width,
                height: text_area.height,
            };
            if let Some(range) = app.selection.paint_range(&geo) {
                let highlighted = apply_selection_highlight(text, range, text_area.width);
                frame.render_widget(Paragraph::new(highlighted), text_area);
            } else {
                frame.render_widget(Paragraph::new(text), text_area);
            }
        } else {
            frame.render_widget(Paragraph::new(text), text_area);
        }
    }

    // Place the single terminal cursor from the real caret index. `TextArea::cursor()`
    // returns `(row, col)` where `col` is a CHARACTER index, but the terminal cursor
    // needs a display COLUMN — and CJK/wide characters occupy two cells each. So the
    // screen column is the display width of the text *before* the caret, not the char
    // count (using the char count put the caret at ~half position for Chinese input).
    // `Line::width()` is unicode-width-aware, so we measure the prefix substring.
    let (row, col) = composer.cursor();
    let prefix_width = lines.get(row).map_or(0, |line| {
        let prefix: String = line.chars().take(col).collect();
        Line::from(prefix).width()
    });
    let cursor_x = text_area
        .x
        .saturating_add(u16::try_from(prefix_width).unwrap_or(u16::MAX))
        .min(text_area.x + text_area.width.saturating_sub(1));
    let cursor_y = text_area
        .y
        .saturating_add(u16::try_from(row).unwrap_or(u16::MAX))
        .min(text_area.y + text_area.height.saturating_sub(1));
    frame.set_cursor_position(Position::new(cursor_x, cursor_y));
}

/// Frames of the animated spinner (activity line + live transcript prefixes).
const WAITING_SPINNER_FRAMES: [&str; 6] = ["✶", "✸", "✹", "✺", "✹", "✷"];

/// Render the live activity line in the second row of the active-turn area,
/// directly above the composer. The first row remains blank to separate this
/// prompt from the transcript above. Carries the four "is it alive / how long /
/// how much / how do I stop it" signals in one glanceable line:
/// `⠹ Working... · 3s · ↑1.2k tok · esc/Ctrl+C to interrupt`. The verb letters
/// take a two-tone marquee highlight (same motion as streaming transcript labels).
/// Everything it needs is pure state on `TuiApp` (spinner frame, elapsed seconds,
/// token count), so it is deterministic under test and never calls the clock itself.
fn render_activity_line(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let Some(label) = app.turn_state().label() else {
        return;
    };
    let theme = app.theme;
    let accent = waiting_indicator_style(theme);
    let meta = transcript_meta_style(theme);

    let frame_index = app.spinner_frame % WAITING_SPINNER_FRAMES.len();
    let spinner = WAITING_SPINNER_FRAMES[frame_index];

    // Spinner glyph + marquee verb. The ellipsis rides with the word so
    // "Working..." / "Responding..." read as one progressive-tense unit.
    // Progressive tense always uses ASCII "...", never the unicode ellipsis.
    let mut spans = vec![
        Span::raw(" "),
        Span::styled(spinner, accent),
        Span::raw(" "),
    ];
    spans.extend(marquee_spans(
        &format!("{label}..."),
        accent,
        app.spinner_frame,
    ));
    spans.push(Span::styled(format!(" · {}s", app.elapsed_seconds), meta));
    if let Some(used) = app.status.context.used_tokens {
        spans.push(Span::styled(
            format!(" · ↑{} tok", format_token_count(used)),
            meta,
        ));
    }
    spans.push(Span::styled(" · esc/Ctrl+C to interrupt", meta));

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Render the transient toast (destination (4) of the noise taxonomy) as a single
/// centered line at the bottom of `area` (the pre-composer gap, or the primary
/// area as a fallback). Success notes are bright/bold; warnings keep a yellow
/// accent. Never a red error chip — those go to the transcript.
fn render_toast(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let Some(toast) = &app.toast else {
        return;
    };
    if area.height == 0 || area.width == 0 {
        return;
    }
    let palette = Palette::for_theme(app.theme);
    // Success notes (e.g. "Copied to clipboard!") use the bright activity accent
    // so they read as a clear confirmation above the composer. Warnings stay
    // yellow. Neither is ever the red reserved for real transcript errors.
    let style = match toast.severity {
        LocalDiagnosticSeverity::Warning => palette.semantic(Semantic::Warning),
        _ => palette
            .semantic(Semantic::Activity)
            .add_modifier(Modifier::BOLD),
    };
    // Draw on the last row of the host area (gap row above the composer when the
    // layout provides one), centered.
    let line_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
    frame.render_widget(Clear, line_area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(toast.message.clone(), style)))
            .alignment(Alignment::Center),
        line_area,
    );
}

fn waiting_indicator_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).semantic(Semantic::Activity)
}

fn slash_suggestion_height(suggestions: &SlashSuggestionState) -> u16 {
    if !suggestions.visible() {
        return 0;
    }
    if suggestions.matches.is_empty() {
        1
    } else {
        suggestions.matches.len().min(7) as u16
    }
}

fn render_slash_suggestions(
    frame: &mut Frame<'_>,
    suggestions: &SlashSuggestionState,
    theme: TerminalTheme,
    area: Rect,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let lines = if suggestions.matches.is_empty() {
        vec![Line::from(Span::styled(
            format!(" no command matches /{}", suggestions.prefix),
            transcript_meta_style(theme),
        ))]
    } else {
        suggestions
            .matches
            .iter()
            .take(usize::from(area.height))
            .map(|item| {
                let marker = if item.selected { "❯" } else { " " };
                let style = if item.selected {
                    slash_selected_style(theme)
                } else {
                    transcript_meta_style(theme)
                };
                Line::from(vec![
                    Span::styled(format!("{marker} /{}", item.name), style),
                    Span::raw("  "),
                    Span::styled(item.summary.clone(), transcript_meta_style(theme)),
                ])
            })
            .collect()
    };
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_status(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    if area.width == 0 {
        return;
    }
    let pending = app.pending_approval_count();
    let status_style = theme_status_style(app.theme);
    let right_style = if pending > 0 {
        approval_pending_style(app.theme)
    } else {
        status_style
    };
    let packed = pack_status_footer(app, area.width, pending, status_style);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(packed.left_width),
            Constraint::Length(packed.gap),
            Constraint::Length(packed.right_width),
        ])
        .split(area);
    frame.render_widget(Paragraph::new(packed.left), horizontal[0]);
    frame.render_widget(
        Paragraph::new(packed.right)
            .alignment(Alignment::Right)
            .style(right_style),
        horizontal[2],
    );
}

/// One packed status-footer row: left chrome, right chrome, and the column widths
/// they occupy (plus the 1-cell gap between them).
struct PackedStatusFooter {
    left: Line<'static>,
    right: String,
    left_width: u16,
    gap: u16,
    right_width: u16,
}

/// Content-aware footer packing.
///
/// Priority when the row is tight:
/// 1. Keep effort + permission mode (always).
/// 2. Keep the full right side (context meter + workspace; pending approval
///    banner always stays visible).
/// 3. Drop connection, then drop provider/model — never compress workspace just
///    to keep a long `provider / model` label.
/// 4. Only if the minimal left + full right still overflows: give left its
///    natural width and let the right column clip (Paragraph truncates).
///
/// Pending approvals prefer keeping provider/model so the left still names the
/// session while the right screams about the gate; they still yield if the row
/// is genuinely too narrow for both.
fn pack_status_footer(
    app: &TuiApp,
    total_width: u16,
    pending: usize,
    status_style: Style,
) -> PackedStatusFooter {
    let connection = ConnectionState::parse(&app.status.connection);
    let effort_style = composer_border_style(app.theme, app.status.reasoning_effort);
    let effort_label = app.status.reasoning_effort.label();
    let mode_label = app.permission_mode.label();
    let provider_prefix = format!("{} / {} · ", app.status.provider, app.status.model);
    let workspace = crate::welcome::short_workspace_path(&app.status.workspace);
    let context_meter = app.status.context.render_bar();

    let right = if pending > 0 {
        format!(
            "⚠ {} approval{} pending · {}  {}",
            pending,
            if pending == 1 { "" } else { "s" },
            context_meter,
            workspace
        )
    } else {
        format!("{context_meter}  {workspace}")
    };
    let right_natural = str_display_width(&right);

    // Left candidates, richest → sparsest. Connection is the first optional
    // segment; provider/model is the second. Effort + mode is the floor.
    let left_full = status_left_line(
        Some(provider_prefix.as_str()),
        effort_label,
        Some(connection.label()),
        mode_label,
        status_style,
        effort_style,
    );
    let left_with_provider = status_left_line(
        Some(provider_prefix.as_str()),
        effort_label,
        None,
        mode_label,
        status_style,
        effort_style,
    );
    let left_minimal = status_left_line(
        None,
        effort_label,
        None,
        mode_label,
        status_style,
        effort_style,
    );

    // Preferred order depends on whether a safety gate is pending.
    let candidates: &[&Line<'static>] = if pending > 0 {
        // Prefer naming the session; still allow dropping provider if needed.
        &[&left_with_provider, &left_minimal]
    } else {
        &[&left_full, &left_with_provider, &left_minimal]
    };

    const GAP: u16 = 1;
    let gap = if total_width == 0 {
        0
    } else {
        GAP.min(total_width)
    };

    // First pass: keep the full right side; pick the richest left that fits.
    for candidate in candidates {
        let left_w = line_display_width_u16(candidate);
        let needed = left_w.saturating_add(gap).saturating_add(right_natural);
        if needed <= total_width {
            // Extra space goes to the left column so the right stays right-aligned
            // against the trailing edge (Paragraph::Right inside its rect).
            let right_width = right_natural.min(total_width.saturating_sub(gap));
            let left_width = total_width.saturating_sub(gap).saturating_sub(right_width);
            return PackedStatusFooter {
                left: (*candidate).clone(),
                right,
                left_width,
                gap,
                right_width,
            };
        }
    }

    // Second pass: nothing fits with a full right. Keep the sparsest left at its
    // natural width (or whatever remains) and give the rest to the right so the
    // workspace/meter compress only after provider/model has already been dropped.
    let left = left_minimal;
    let left_natural = line_display_width_u16(&left);
    let left_width = left_natural.min(total_width.saturating_sub(gap));
    let right_width = total_width.saturating_sub(gap).saturating_sub(left_width);
    PackedStatusFooter {
        left,
        right,
        left_width,
        gap,
        right_width,
    }
}

fn status_left_line(
    provider_prefix: Option<&str>,
    effort_label: &'static str,
    connection: Option<&'static str>,
    mode_label: &'static str,
    status_style: Style,
    effort_style: Style,
) -> Line<'static> {
    let mut spans = Vec::with_capacity(4);
    if let Some(prefix) = provider_prefix {
        spans.push(Span::styled(prefix.to_string(), status_style));
    }
    spans.push(Span::styled(effort_label, effort_style));
    let tail = match connection {
        Some(conn) => format!(" · {conn} · {mode_label}"),
        None => format!(" · {mode_label}"),
    };
    spans.push(Span::styled(tail, status_style));
    Line::from(spans)
}

fn str_display_width(text: &str) -> u16 {
    u16::try_from(unicode_width::UnicodeWidthStr::width(text)).unwrap_or(u16::MAX)
}

fn line_display_width_u16(line: &Line<'_>) -> u16 {
    u16::try_from(line.width()).unwrap_or(u16::MAX)
}

/// Style for the footer's pending-approval indicator — warning-colored so a
/// pending safety gate stands out from the neutral status text.
fn approval_pending_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).semantic(Semantic::Attention)
}

/// Compute a centered rectangle `pct_x` percent wide and exactly `height` rows
/// tall (clamped to the area), used to anchor a modal whose height auto-fits its
/// content rather than a fixed percentage.
fn centered_rect_with_height(pct_x: u16, height: u16, area: Rect) -> Rect {
    let height = height.min(area.height);
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(Rect::new(area.x, y, area.width, height));
    horizontal[1]
}

/// Expand a modal by one cell on every available side, clipped to the frame.
/// Clearing this outer ring keeps wide CJK glyphs and underlying styled cells
/// from visually colliding with the border without shrinking modal content.
fn modal_clear_area(modal: Rect, bounds: Rect) -> Rect {
    let left = modal.x.saturating_sub(1).max(bounds.x);
    let top = modal.y.saturating_sub(1).max(bounds.y);
    let right = modal.right().saturating_add(1).min(bounds.right());
    let bottom = modal.bottom().saturating_add(1).min(bounds.bottom());
    Rect::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
}

fn modal_style(theme: TerminalTheme, role: Semantic) -> Style {
    Palette::for_theme(theme).semantic(role)
}

/// Build the content lines for the bottom-docked approval panel (Claude Code
/// style). Shared by height calculation and rendering so the reserved slot
/// always matches what is drawn.
///
/// Layout:
/// ```text
/// ──────────────────────────────────
/// Approval required
/// Shell command cargo test --all
/// …
/// ```
/// Color rule: the top rule uses amber; the *selected* option text uses amber
/// fg; the concrete command/path after the kind label is muted; other copy is
/// body/meta color.
fn approval_panel_lines(app: &TuiApp) -> Vec<Line<'static>> {
    let theme = app.theme;
    let accent = approval_accent_style(theme);
    let body = Style::default();
    let meta = transcript_meta_style(theme);

    // Content width inside the horizontal gutter — used for the top rule and
    // for truncating the concrete command/path on the kind line.
    let content_width = usize::from(
        app.terminal_size
            .width
            .saturating_sub(transcript_horizontal_gutter(app.terminal_size.width).saturating_mul(2))
            .max(16),
    );

    let mut lines: Vec<Line<'static>> = Vec::new();
    // Full-width top rule.
    lines.push(Line::from(Span::styled("─".repeat(content_width), accent)));

    let Some(request) = app.approval_panel.focused_request() else {
        lines.push(Line::from(Span::styled(
            "Approval required",
            body.add_modifier(Modifier::BOLD),
        )));
        return lines;
    };

    let pending = app.approval_panel.pending.len();
    let focus = app.approval_panel.focused_index.unwrap_or(0) + 1;
    let title = if pending > 1 {
        format!("Approval {focus}/{pending}")
    } else {
        "Approval required".to_string()
    };
    lines.push(Line::from(Span::styled(
        title,
        body.add_modifier(Modifier::BOLD),
    )));

    // Kind + concrete target on one line: "Shell command cargo test --all".
    // Shell commands get token-level syntax highlight; other targets stay muted.
    let kind = approval_operation_label(request);
    if let Some(detail) = approval_operation_detail(request) {
        let kind_cols = Line::from(format!("{kind} ")).width();
        let detail_max = content_width.saturating_sub(kind_cols).max(8);
        let detail = truncate_display_text(&detail, detail_max);
        let mut row = vec![Span::styled(
            format!("{kind} "),
            body.add_modifier(Modifier::BOLD),
        )];
        if request.preview.kind == "command"
            || request.preview.resource.contains("shell_command")
            || request.resource.contains("shell_command")
        {
            row.extend(crate::shell_highlight::highlight_shell_command(
                &detail, theme,
            ));
        } else {
            row.push(Span::styled(detail, meta));
        }
        lines.push(Line::from(row));
    } else {
        lines.push(Line::from(Span::styled(
            kind,
            body.add_modifier(Modifier::BOLD),
        )));
    }

    // Optional human reason, but skip policy boilerplate that adds no signal.
    // File-edit approvals stay compact (path + choices only): the full colored
    // diff is shown under Wrote/Edited in the transcript after apply, not here.
    if !request.reason.is_empty()
        && !is_boilerplate_approval_text(&request.reason)
        && request.preview.kind != "file_edit"
    {
        lines.push(Line::from(Span::styled(request.reason.clone(), meta)));
    }

    // Truncation / redaction hints only for non-edit previews (edits no longer
    // dump a multi-line diff body into the approval surface).
    if request.preview.kind != "file_edit" {
        if request.preview.truncated {
            lines.push(Line::from(Span::styled("… (truncated)", meta)));
        }
        if request.preview.redacted {
            lines.push(Line::from(Span::styled("(redacted)", meta)));
        }
    }

    lines.push(Line::from(Span::raw("")));

    if let Some(reject) = &app.reject_reason {
        lines.push(Line::from(Span::styled(
            "Reject & redirect — type a reason, Enter to send, Esc to cancel:",
            body,
        )));
        lines.push(Line::from(Span::styled(format!("❯ {}", reject.text), body)));
    } else {
        // Three choices. Only the highlighted row uses the amber accent (fg +
        // bold, no reverse/background). Unselected rows stay body-colored.
        for (option, label) in [
            (ApprovalOption::AllowOnce, "1  Allow once"),
            (
                ApprovalOption::AllowAlways,
                "2  Allow always (this tool + path, this session)",
            ),
            (ApprovalOption::Reject, "3  Reject & redirect"),
        ] {
            let selected = app.approval_selection == option;
            let marker = if selected { "❯ " } else { "  " };
            let style = if selected {
                accent.add_modifier(Modifier::BOLD)
            } else {
                body
            };
            lines.push(Line::from(Span::styled(format!("{marker}{label}"), style)));
        }
        // Blank row between choices and the muted key-binding hint.
        lines.push(Line::from(Span::raw("")));
        // Operation hints stay muted so they don't compete with the decision rows.
        lines.push(Line::from(Span::styled(
            "↑/↓ select · Enter confirm · Tab next request · Esc dismiss (stays pending)",
            meta,
        )));
    }

    lines
}

/// Friendly operation label shown above the concrete command/path.
fn approval_operation_label(request: &ruse_ui_bridge::UiApprovalRequest) -> String {
    match request.preview.kind.as_str() {
        "command" => "Shell command".to_string(),
        "file_edit" => "Edit file".to_string(),
        "git" => "Git".to_string(),
        "mcp" => "MCP tool".to_string(),
        "plugin" => "Plugin".to_string(),
        "background_task" => "Background task".to_string(),
        "generic" | "tool" => friendly_tool_kind_label(&request.preview.resource)
            .or_else(|| friendly_tool_kind_label(&request.resource))
            .unwrap_or_else(|| "Tool".to_string()),
        other if !other.trim().is_empty() => humanize_kind_label(other),
        _ => "Permission".to_string(),
    }
}

fn friendly_tool_kind_label(name: &str) -> Option<String> {
    Some(match name {
        "workspace.shell_command" => "Shell command".to_string(),
        "workspace.write_file" => "Write file".to_string(),
        "workspace.edit_file" => "Edit file".to_string(),
        "workspace.read_file" => "Read file".to_string(),
        "workspace.search_text" => "Search".to_string(),
        "workspace.git_status" => "Git status".to_string(),
        "workspace.git_diff" => "Git diff".to_string(),
        other if other.starts_with("mcp.") => {
            format!("MCP {}", other.trim_start_matches("mcp."))
        }
        other if other.starts_with("plugin.") => {
            format!("Plugin {}", other.trim_start_matches("plugin."))
        }
        _ => return None,
    })
}

fn humanize_kind_label(kind: &str) -> String {
    let words = kind.replace('_', " ");
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Permission".to_string(),
    }
}

/// Concrete operation target: command line, path, or other preview resource.
/// Skips tool-name-only values and boilerplate summaries like
/// "Approve tool execution for workspace.shell_command".
///
/// For `file_edit`, only a single path line is returned — never the multi-line
/// diff summary (that body belongs under Wrote/Edited after apply).
fn approval_operation_detail(request: &ruse_ui_bridge::UiApprovalRequest) -> Option<String> {
    let is_file_edit = request.preview.kind == "file_edit";
    let candidates = if is_file_edit {
        // Path only — do not fall through to preview.summary (often a full diff).
        vec![request.preview.resource.as_str(), request.resource.as_str()]
    } else {
        vec![
            request.preview.resource.as_str(),
            request.resource.as_str(),
            request.preview.summary.as_str(),
        ]
    };
    for candidate in candidates {
        let trimmed = candidate.trim();
        if trimmed.is_empty() || is_tool_name_only(trimmed) || is_boilerplate_approval_text(trimmed)
        {
            continue;
        }
        // Prefer the bare command over "Run <cmd> in … with timeout …".
        if let Some(command) = strip_run_command_prefix(trimmed) {
            return Some(command);
        }
        // Keep approval one-liners compact even if a summary sneaks in.
        if trimmed.contains('\n') {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

fn is_tool_name_only(value: &str) -> bool {
    !value.contains(' ')
        && (value.starts_with("workspace.")
            || value.starts_with("mcp.")
            || value.starts_with("plugin.")
            || value.starts_with("background."))
}

fn is_boilerplate_approval_text(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    lower.starts_with("approve tool execution")
        || lower == "tool execution requested"
        || lower == "tool execution requires approval"
        || lower.ends_with(" requires approval")
}

/// "Run cargo test --all in . with timeout 30000ms" → "cargo test --all".
fn strip_run_command_prefix(summary: &str) -> Option<String> {
    let rest = summary.strip_prefix("Run ")?;
    if let Some((command, _)) = rest.split_once(" in ") {
        let command = command.trim();
        if !command.is_empty() && !is_tool_name_only(command) {
            return Some(command.to_string());
        }
    }
    let command = rest.trim();
    (!command.is_empty() && !is_tool_name_only(command)).then(|| command.to_string())
}

/// Truncate a string to at most `max_cols` terminal display columns, appending
/// `…` when clipped. Uses unicode display width so CJK is counted correctly.
fn truncate_display_text(text: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    if Line::from(text).width() <= max_cols {
        return text.to_string();
    }
    if max_cols == 1 {
        return "…".to_string();
    }
    let budget = max_cols.saturating_sub(1);
    let mut out = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let ch_width = Line::from(ch.to_string()).width();
        if used + ch_width > budget {
            break;
        }
        out.push(ch);
        used += ch_width;
    }
    out.push('…');
    out
}

/// Height of the bottom-docked approval surface: pure content rows (badge +
/// body), capped so a long preview cannot swallow the transcript. While visible
/// it also covers the status footer, so the surface may use the full bottom of
/// the terminal (leaving only a 1-row transcript min).
///
/// Shared with hit-testing so clickable transcript geometry matches the live
/// layout when approval replaces the composer.
#[must_use]
pub(crate) fn approval_panel_height_for(app: &TuiApp, terminal_height: u16) -> u16 {
    let content_height = u16::try_from(approval_panel_lines(app).len()).unwrap_or(u16::MAX);
    let desired = content_height.max(5);
    // Transcript min 1 row; no status footer while approval is up.
    let max = terminal_height.saturating_sub(1).max(5);
    desired.min(max)
}

/// Render the approval decision surface in the bottom input slot (replacing the
/// composer and covering the status footer). No box borders — the title is a
/// reversed amber badge on the first content row.
fn render_approval_panel(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let lines = approval_panel_lines(app);
    if area.width == 0 || area.height == 0 {
        return;
    }

    let content_area = area.inner(Margin {
        horizontal: transcript_horizontal_gutter(area.width),
        vertical: 0,
    });
    if content_area.width == 0 || content_area.height == 0 {
        return;
    }
    frame.render_widget(Paragraph::new(lines), content_area);

    // Place the terminal cursor on the reject-reason input when capturing a
    // denial reason so typing feels like the normal composer.
    if let Some(reject) = &app.reject_reason {
        let prefix = format!("❯ {}", reject.text);
        let cursor_col = u16::try_from(Line::from(prefix.as_str()).width()).unwrap_or(u16::MAX);
        // Reason line is the last content row.
        let reason_row = content_area.height.saturating_sub(1);
        let cursor_x = content_area
            .x
            .saturating_add(cursor_col)
            .min(content_area.x + content_area.width.saturating_sub(1));
        let cursor_y = content_area.y.saturating_add(reason_row);
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }
}

/// Amber accent used for the approval top rule and the currently selected option
/// text. Theme-aware via [`Semantic::ApprovalModal`].
fn approval_accent_style(theme: TerminalTheme) -> Style {
    modal_style(theme, Semantic::ApprovalModal)
}

/// Render the theme picker as a centered overlay. Each row is one
/// [`TerminalTheme`]; the highlight is the live preview (applied by the reducer
/// on Up/Down). Enter persists, Esc restores the theme that was active when the
/// picker opened.
fn render_theme_modal(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    use crate::ThemePickerState;

    let picker = &app.theme_picker;
    let theme = app.theme;
    let accent = modal_style(theme, Semantic::ModelModal);
    let focused = picker.focused_index;

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Select a theme (live preview)",
        accent,
    )));
    lines.push(Line::from(Span::raw("")));

    for (index, candidate) in ThemePickerState::THEMES.iter().enumerate() {
        let selected = index == focused;
        let is_current = *candidate == picker.original_theme;
        let marker = if selected { "❯ " } else { "  " };
        let current_tag = if is_current { " (was)" } else { "" };
        let label = format!("{marker}{}{current_tag}", candidate.name());
        // Selected row: accent + bold only — no reverse/background fill.
        let style = if selected {
            accent.add_modifier(Modifier::BOLD)
        } else {
            theme_status_style(theme)
        };
        lines.push(Line::from(Span::styled(label, style)));
    }

    lines.push(Line::from(Span::styled(
        "↑/↓ preview · Enter confirm · Esc cancel",
        transcript_meta_style(theme),
    )));

    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let max_height = area.height.saturating_sub(2).max(3);
    let modal_height = content_height.saturating_add(2).min(max_height);
    let modal_area = centered_rect_with_height(60, modal_height, area);
    frame.render_widget(Clear, modal_clear_area(modal_area, area));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(accent)
        .title_style(accent)
        .title(" Theme ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the `/status` popup as a centered read-only overlay (same shape as the
/// model/session/theme modals). Enter or Esc dismisses it.
fn render_status_modal(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let theme = app.theme;
    let accent = modal_style(theme, Semantic::ModelModal);
    let snapshot = &app.status_panel.snapshot;

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled("Session status", accent)));
    lines.push(Line::from(Span::raw("")));

    let body = theme_status_style(theme);
    let label = transcript_meta_style(theme);
    let mut field = |name: &str, value: String| {
        lines.push(Line::from(vec![
            Span::styled(format!("{name}: "), label),
            Span::styled(value, body),
        ]));
    };
    field("Connection", snapshot.connection.clone());
    field(
        "Provider",
        format!("{} / {}", snapshot.provider, snapshot.model),
    );
    field("Workspace", snapshot.workspace.clone());
    field("Mode", snapshot.mode.clone());
    field("Sessions", snapshot.session_count.to_string());
    field("Diagnostics", snapshot.diagnostic_count.to_string());

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "Enter/Esc close",
        transcript_meta_style(theme),
    )));

    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let max_height = area.height.saturating_sub(2).max(3);
    let modal_height = content_height.saturating_add(2).min(max_height);
    let modal_area = centered_rect_with_height(60, modal_height, area);
    frame.render_widget(Clear, modal_clear_area(modal_area, area));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(accent)
        .title_style(accent)
        .title(" Status ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_model_modal(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let picker = &app.model_picker;
    if picker.entries.is_empty() {
        return;
    }
    let theme = app.theme;
    let accent = modal_style(theme, Semantic::ModelModal);
    let focused = picker.focused_index.unwrap_or(0);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled("Select a model provider", accent)));
    lines.push(Line::from(Span::raw("")));

    for (index, entry) in picker.entries.iter().enumerate() {
        let selected = index == focused;
        let is_current = entry.provider_id == picker.selected_provider;
        let marker = if selected { "❯ " } else { "  " };
        let current_tag = if is_current { " (current)" } else { "" };
        let disabled_tag = if entry.enabled { "" } else { " · disabled" };
        let label = format!(
            "{marker}{} / {}{current_tag}{disabled_tag}",
            entry.provider_id, entry.model_id
        );
        // Selected row: accent + bold only — no reverse/background fill.
        let style = if selected {
            accent.add_modifier(Modifier::BOLD)
        } else if !entry.enabled {
            transcript_meta_style(theme)
        } else {
            theme_status_style(theme)
        };
        lines.push(Line::from(Span::styled(label, style)));
    }

    // Blank row separates the provider list from the muted operation hint.
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "↑/↓ select · Enter switch · Esc cancel",
        transcript_meta_style(theme),
    )));

    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let max_height = area.height.saturating_sub(2).max(3);
    let modal_height = content_height.saturating_add(2).min(max_height);
    let modal_area = centered_rect_with_height(80, modal_height, area);
    frame.render_widget(Clear, modal_clear_area(modal_area, area));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(accent)
        .title_style(accent)
        .title(" Model ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the session picker as a centered overlay (same shape as the model
/// modal). It lists sessions for the current workspace; Enter switches to the
/// highlighted row and closes the picker. Create a fresh session with `/new`.
fn render_session_modal(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let panel = &app.session_panel;
    let theme = app.theme;
    let accent = modal_style(theme, Semantic::SessionModal);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled("Switch session", accent)));
    lines.push(Line::from(Span::raw("")));

    if panel.sessions.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No sessions in this workspace",
            theme_status_style(theme),
        )));
    } else {
        for (index, session) in panel.sessions.iter().enumerate() {
            let selected = panel.focused_index == Some(index);
            let marker = if selected { "❯ " } else { "  " };
            let label = format!(
                "{marker}{} · {} · {}/{}",
                session.status, session.title, session.provider_id, session.model_id
            );
            // Selected row: accent + bold only — no reverse/background fill.
            let style = if selected {
                accent.add_modifier(Modifier::BOLD)
            } else {
                theme_status_style(theme)
            };
            lines.push(Line::from(Span::styled(label, style)));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "↑/↓ select · Enter switch · Esc cancel · /new for a fresh session",
        transcript_meta_style(theme),
    )));

    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let max_height = area.height.saturating_sub(2).max(3);
    let modal_height = content_height.saturating_add(2).min(max_height);
    let modal_area = centered_rect_with_height(80, modal_height, area);
    frame.render_widget(Clear, modal_clear_area(modal_area, area));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(accent)
        .title_style(accent)
        .title(" Sessions ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

fn composer_border_style(theme: TerminalTheme, effort: crate::ReasoningEffort) -> Style {
    Palette::for_theme(theme).composer_border(effort)
}

fn theme_status_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).status_bar()
}

fn transcript_meta_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).meta()
}

/// Softer muted style for Thinking/Thought stats + expand hints.
/// Uses the same slate muted foreground as Reasoning, not terminal Gray/Meta,
/// so `[0s · 98 tokens] (click…)` stays secondary until hover brightens it.
/// Stats stay upright and un-bolded so the status word carries the weight;
/// only the expanded reasoning body is italic.
fn reasoning_detail_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme)
        .semantic(Semantic::Reasoning)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC)
}

/// Selected slash-suggestion row: accent fg + bold, no reverse/background fill
/// (matches model/session/approval picker selection language).
fn slash_selected_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme)
        .semantic(Semantic::Activity)
        .add_modifier(Modifier::BOLD)
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut output = String::new();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            output.push_str(buffer[(x, y)].symbol());
        }
        output.push('\n');
    }
    output
}

/// All wrapped/padded transcript rows in document order, before viewport
/// scrolling. Cross-screen selection stores absolute row coordinates against
/// this list so scrolling never changes which text the anchor refers to.
#[must_use]
pub(crate) fn full_transcript_plain_lines_for_selection(
    app: &TuiApp,
    content_width: u16,
) -> Vec<String> {
    if app.transcript.items.is_empty() || content_width == 0 {
        return Vec::new();
    }
    let lines = transcript_styled_lines(
        &app.transcript.items,
        app.theme,
        app.spinner_frame,
        app.markdown_enabled,
        usize::from(content_width),
        app.hovered_transcript_item(),
    );
    let wrapped = wrap_transcript_lines(lines, content_width);
    pad_block_backgrounds(wrapped, content_width)
        .into_iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect()
}

/// Plain-text rows currently painted in the transcript content area.
/// Must stay identical to [`render_transcript`]'s wrap → pad → scroll pipeline
/// so in-app selection copies what the user sees.
#[must_use]
pub(crate) fn visible_transcript_plain_lines_for_selection(
    app: &TuiApp,
    content_width: u16,
    content_height: u16,
) -> Vec<String> {
    if content_height == 0 {
        return Vec::new();
    }
    let full = full_transcript_plain_lines_for_selection(app, content_width);
    let start = transcript_visible_start(
        full.len(),
        usize::from(content_height),
        app.transcript.scroll_offset,
    );
    let end = (start + usize::from(content_height)).min(full.len());
    let mut out = full[start..end].to_vec();
    let target = usize::from(content_height);
    while out.len() < target {
        out.push(String::new());
    }
    out
}

pub fn sticky_chrome_height(app: &crate::TuiApp) -> u16 {
    // Bottom input slot (composer ~3 or approval panel content). While approval
    // is up it covers the status footer, so no extra footer row is reserved.
    let input_height = if app.approval_modal_visible() {
        approval_panel_height_for(app, app.terminal_size.height)
    } else {
        composer_height_for(app).saturating_add(1) // + status footer
    };
    input_height
        .max(3)
        .min(app.terminal_size.height.saturating_sub(2).max(3))
}

pub fn render_sticky_chrome(frame: &mut ratatui::Frame<'_>, app: &crate::TuiApp) {
    render_app(frame, app);
}

#[must_use]
pub fn transcript_history_lines(app: &crate::TuiApp, width: u16) -> Vec<String> {
    app.transcript
        .items
        .iter()
        .filter(|item| !item.streaming)
        .map(|item| {
            let line = item.render_line();
            if width == 0 || line.chars().count() <= width as usize {
                line
            } else {
                line.chars().take(width as usize).collect()
            }
        })
        .collect()
}

pub fn paint_transcript_history_lines(
    buf: &mut ratatui::buffer::Buffer,
    lines: &[String],
    width: u16,
) {
    let w = width.max(1) as usize;
    for (row, line) in lines.iter().enumerate() {
        if row as u16 >= buf.area.height {
            break;
        }
        let truncated: String = line.chars().take(w).collect();
        buf.set_stringn(
            0,
            row as u16,
            truncated,
            w,
            ratatui::style::Style::default(),
        );
    }
}
