use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{Frame, Terminal};

use crate::app::format_token_count;
use crate::{
    ApprovalOption, ConnectionState, LocalDiagnosticSeverity, Palette, Semantic,
    SlashSuggestionState, TerminalTheme, TranscriptItem, TranscriptItemKind, TuiApp, TuiScreen,
};

pub fn render_app(frame: &mut Frame<'_>, app: &TuiApp) {
    match app.screen {
        TuiScreen::Chat => render_chat(frame, app),
        TuiScreen::Help => render_help(frame, app),
    }
}

pub fn render_app_to_string(app: &TuiApp, width: u16, height: u16) -> std::io::Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render_app(frame, app))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

fn render_chat(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let suggestion_height = slash_suggestion_height(&app.composer.slash_suggestions);
    // Active turns reserve one blank row between the transcript and the activity
    // prompt, then one row for the prompt itself. When idle the pair collapses so
    // the transcript reclaims both rows.
    let activity_height: u16 = if app.turn_state().is_active() { 2 } else { 0 };

    // Composer height grows with content: the two border rows plus one row per
    // logical line, capped so a long paste can't swallow the transcript. Wrapping
    // is off, so one logical line = one screen row.
    let composer_height = composer_height_for(app);

    // Bottom-anchored stack: transcript (flex) → blank row + activity line →
    // [slash suggestions] → composer → status footer. Keeping suggestions next
    // to the composer means they never interrupt transcript/activity spacing.
    let mut constraints = vec![Constraint::Min(3)];
    if activity_height > 0 {
        constraints.push(Constraint::Length(activity_height));
    }
    if suggestion_height > 0 {
        constraints.push(Constraint::Length(suggestion_height));
    }
    constraints.push(Constraint::Length(composer_height)); // composer
    constraints.push(Constraint::Length(1)); // status footer
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Walk the split rects in the same order the constraints were pushed.
    let primary_area = vertical[0];
    let mut idx = 1;
    let activity_area = if activity_height > 0 {
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
    let composer_area = vertical[idx];
    let status_area = vertical[idx + 1];

    // Side panels (status/trace/tasks/extensions/diff/command) are intentionally
    // not drawn: the product shell is a single full-width transcript like Claude
    // Code. Status lives in the footer; model/session/theme use centered modals.
    render_primary(frame, app, primary_area);

    if let Some(activity_area) = activity_area {
        let prompt_area = Rect::new(
            activity_area.x,
            activity_area.y.saturating_add(1),
            activity_area.width,
            activity_area.height.saturating_sub(1),
        );
        render_activity_line(frame, app, prompt_area);
    }
    if let Some(suggestion_area) = suggestion_area {
        render_slash_suggestions(
            frame,
            &app.composer.slash_suggestions,
            app.theme,
            suggestion_area,
        );
    }
    render_composer(frame, app, composer_area);
    render_status(frame, app, status_area);

    // The transient toast (destination (4)) is a dim overlay on the last row of
    // the primary area — neutral, never a red chip. It auto-expires (the event
    // loop clears it), so it stays out of the transcript's permanent record.
    if app.toast.is_some() {
        render_toast(frame, app, primary_area);
    }

    // The approval modal is drawn LAST, over the whole chat area, so it is always
    // visible when a request is pending regardless of terminal width or which side
    // panel is active. It is a true overlay (Clear + centered bordered block).
    if app.approval_modal_visible() {
        render_approval_modal(frame, app, area);
    }

    // The model picker is a centered overlay, drawn last so it sits above the
    // chat. It never coexists with the approval modal (opening one is a
    // deliberate, separate user action), but the approval gate wins if both are
    // somehow active so a pending safety decision is never hidden.
    if app.model_picker.is_open() && !app.approval_modal_visible() {
        render_model_modal(frame, app, area);
    }

    // The session picker is the same centered-overlay shape. The approval gate
    // still wins if both are somehow active, and the model picker takes priority
    // over it (they are mutually exclusive in practice, but this keeps a single
    // deterministic winner).
    if app.session_panel.is_open() && !app.approval_modal_visible() && !app.model_picker.is_open() {
        render_session_modal(frame, app, area);
    }

    // Theme picker: same centered-overlay shape. Approval still wins; model and
    // session take priority so a deliberate switch is never buried under theme.
    if app.theme_picker.is_open()
        && !app.approval_modal_visible()
        && !app.model_picker.is_open()
        && !app.session_panel.is_open()
    {
        render_theme_modal(frame, app, area);
    }
}

fn render_help(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);
    let help = Paragraph::new(
        "agent Help\n\
         Enter sends · Shift+Enter newline\n\
         Up/Down scroll the transcript · Alt+Up/Down prompt history\n\
         Esc interrupts the turn (Ctrl+C again to quit)\n\
         Tab cycles permission mode · Shift+Tab cycles reasoning effort\n\
         Ctrl+P toggles latest thinking · Ctrl+T opens theme picker (/theme)\n\
         Ctrl+G switches model (/model) · Ctrl+S sessions · Ctrl+R recovery",
    )
    .block(theme_block("Help", app.theme));
    frame.render_widget(help, vertical[0]);
    render_status(frame, app, vertical[1]);
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
    let lines = transcript_styled_lines(&transcript.items, theme);
    let wrapped = wrap_transcript_lines(lines, content_area.width);
    // After wrapping, pad every line that carries a block background (user/model
    // message blocks) out to the content width. The outer gutter keeps messages
    // away from the terminal edges while each message still reads as one block.
    let padded = pad_block_backgrounds(wrapped, content_area.width);
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
    if let Some(range) = selection_paint_range(app, content_area) {
        let highlighted = apply_selection_highlight(visible, range, content_area.width);
        frame.render_widget(Paragraph::new(highlighted), content_area);
    } else {
        frame.render_widget(Paragraph::new(visible), content_area);
    }
}

/// Active selection range in content-local coordinates, if any.
fn selection_paint_range(app: &TuiApp, content_area: Rect) -> Option<crate::SelectionRange> {
    let geo = crate::ContentGeometry {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: content_area.height,
    };
    app.selection.paint_range(&geo)
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
    let welcome = Paragraph::new(welcome_text(app)).style(theme_welcome_style(app.theme));
    frame.render_widget(welcome, area);
}

fn welcome_text(app: &TuiApp) -> String {
    let provider_health = provider_health_summary(app);
    let extension_health = extension_health_summary(app);
    let session_hint = if app.session_panel.sessions.is_empty() {
        "new session".to_string()
    } else {
        format!("{} recent sessions", app.session_panel.sessions.len())
    };
    format!(
        "AGENT\n\n{} / {}  {}\ncontext {}\n\nWorkspace\n  {}\n  {} · {}\n\nExtensions\n  {}",
        app.status.provider,
        app.status.model,
        provider_health,
        app.status.context.render(),
        app.status.workspace,
        ConnectionState::parse(&app.status.connection),
        session_hint,
        extension_health
    )
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

fn transcript_styled_lines(items: &[TranscriptItem], theme: TerminalTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut rendered_any = false;
    for item in items {
        let item_lines = transcript_item_lines(item, theme);
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

/// Hit-test twin of [`transcript_item_lines`].
pub(crate) fn transcript_item_lines_for_hit(
    item: &TranscriptItem,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    transcript_item_lines(item, theme)
}

fn transcript_item_lines(item: &TranscriptItem, theme: TerminalTheme) -> Vec<Line<'static>> {
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
            message_block_lines(&body, "", style)
        }
        TranscriptItemKind::Reasoning => reasoning_lines(item, &body, theme),
        TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => {
            tool_call_lines(item, &body, item.kind, theme)
        }
        TranscriptItemKind::Approval => {
            if body_is_empty {
                return Vec::new();
            }
            vec![Line::from(vec![
                Span::styled(" approval ", transcript_chip_style(item.kind, theme)),
                Span::raw(" "),
                Span::styled(body, style),
            ])]
        }
        TranscriptItemKind::Diagnostic => {
            if body_is_empty && item.title.trim().is_empty() {
                return Vec::new();
            }
            vec![
                Line::from(vec![
                    Span::styled(" diagnostic ", transcript_chip_style(item.kind, theme)),
                    Span::raw(" "),
                    Span::styled(item.title.clone(), style),
                ]),
                Line::from(Span::styled(format!("  {body}"), style)),
            ]
        }
        TranscriptItemKind::Progress => {
            if body_is_empty {
                return Vec::new();
            }
            vec![Line::from(vec![
                Span::styled(" note ", transcript_chip_style(item.kind, theme)),
                Span::raw(" "),
                Span::styled(body, style),
            ])]
        }
    }
}

/// Default number of result rows shown under a tool call. Shell output often
/// needs two lines to be useful; anything longer collapses until Ctrl+O expands.
const TOOL_RESULT_DEFAULT_ROWS: usize = 2;

/// Render a tool call as:
///   [tool_name] target          (same line)
///     ⎿ result line 1
///       result line 2
/// Body format is preferably `friendly_name target\nresult…` (legacy
/// `friendly_name target - result` is still accepted).
fn tool_call_lines(
    item: &TranscriptItem,
    body: &str,
    kind: TranscriptItemKind,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    let (head, detail) = match body.split_once('\n') {
        Some((head, detail)) => (head.trim(), detail.trim()),
        None => match body.split_once(" - ") {
            Some((head, detail)) => (head.trim(), detail.trim()),
            None => (body.trim(), ""),
        },
    };

    // Split "friendly_name target…" so the name can be chipped.
    let (tool_label, target) = match head.split_once(' ') {
        Some((name, rest)) => (name, rest.trim()),
        None => (head, ""),
    };
    let chip = transcript_chip_style(TranscriptItemKind::ToolCall, theme);
    let target_style = if kind == TranscriptItemKind::ToolCall {
        transcript_meta_style(theme)
    } else {
        Style::default()
    };

    let mut head_spans = vec![
        Span::styled(format!(" {tool_label} "), chip),
        Span::raw(" "),
    ];
    if !target.is_empty() {
        head_spans.push(Span::styled(target.to_string(), target_style));
    }
    if kind == TranscriptItemKind::ToolCall && item.streaming {
        head_spans.push(Span::styled(" …", transcript_meta_style(theme)));
    }
    let mut lines = vec![Line::from(head_spans)];

    if detail.is_empty() {
        return lines;
    }

    let meta = transcript_meta_style(theme);
    let detail_rows: Vec<&str> = detail
        .lines()
        .filter(|row| !row.trim().is_empty())
        .collect();
    if detail_rows.is_empty() {
        return lines;
    }

    let expanded = item.tool_output_expanded;
    let shown = if expanded {
        detail_rows.len()
    } else {
        detail_rows.len().min(TOOL_RESULT_DEFAULT_ROWS)
    };
    for (index, row) in detail_rows.iter().take(shown).enumerate() {
        let connector = if index == 0 { "  ⎿ " } else { "    " };
        lines.push(Line::from(vec![
            Span::styled(connector, meta),
            Span::styled((*row).to_string(), meta),
        ]));
    }
    if !expanded && detail_rows.len() > shown {
        let hidden = detail_rows.len() - shown;
        lines.push(Line::from(Span::styled(
            format!("    … +{hidden} lines (click / ctrl+o to expand)"),
            meta,
        )));
    }
    lines
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

/// Render a reasoning ("thinking") item as a collapsible block: a clickable-style
/// header (`▾ thinking` / `▸ thinking (N lines)`) plus, when expanded, the
/// reasoning content. Collapsed shows only the header; expanded shows at least the
/// first few lines. The header hint tells the user how to toggle it.
fn reasoning_lines(item: &TranscriptItem, body: &str, theme: TerminalTheme) -> Vec<Line<'static>> {
    let collapsed = item.is_reasoning_collapsed();
    let style = transcript_item_style(TranscriptItemKind::Reasoning, theme);
    let meta = transcript_meta_style(theme);
    let chip = transcript_chip_style(TranscriptItemKind::Reasoning, theme);

    // The reasoning body, minus the pending placeholder sentinel (which is just a
    // "waiting for content" marker, not real reasoning).
    let content: Vec<&str> = if body == "thinking" {
        Vec::new()
    } else {
        body.lines().collect()
    };

    let mut header_spans = vec![Span::styled(" thinking ", chip), Span::raw(" ")];
    if collapsed && !content.is_empty() {
        header_spans.push(Span::styled(
            format!("({} lines · ctrl+p to expand) ▸", content.len()),
            meta,
        ));
    } else if !content.is_empty() {
        header_spans.push(Span::styled("(ctrl+p to collapse) ▾", meta));
    } else {
        // No content yet: streaming placeholder.
        header_spans.push(Span::styled("…", meta));
    }
    let mut output = vec![Line::from(header_spans)];

    if collapsed {
        return output;
    }

    // Expanded: show the reasoning content, indented as an aside.
    for line in &content {
        output.push(Line::from(Span::styled(format!("  {line}"), style)));
    }
    if item.streaming && !content.is_empty() {
        output.push(Line::from(Span::styled("  …", meta)));
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

fn transcript_chip_style(kind: TranscriptItemKind, theme: TerminalTheme) -> Style {
    let palette = Palette::for_theme(theme);
    match kind {
        TranscriptItemKind::Diagnostic => palette.chip(Semantic::Error),
        TranscriptItemKind::Approval => palette.chip(Semantic::Approval),
        TranscriptItemKind::Reasoning => palette.chip(Semantic::Reasoning),
        TranscriptItemKind::ToolCall | TranscriptItemKind::ToolResult => {
            palette.chip(Semantic::Tool)
        }
        _ => palette.meta(),
    }
}

/// Fixed gutter width for the composer prompt marker (`❯ `).
const COMPOSER_GUTTER: u16 = 2;

/// Maximum number of text rows the composer grows to before it stops expanding
/// (a long paste scrolls within the box instead of swallowing the transcript).
const COMPOSER_MAX_TEXT_ROWS: u16 = 6;

/// Dynamic composer height: the two border rows plus one row per logical line,
/// capped at `COMPOSER_MAX_TEXT_ROWS` text rows. Grows as the user types multi-line
/// input, shrinks back when it is cleared.
fn composer_height_for(app: &TuiApp) -> u16 {
    let text_rows = u16::try_from(app.composer.line_count())
        .unwrap_or(COMPOSER_MAX_TEXT_ROWS)
        .clamp(1, COMPOSER_MAX_TEXT_ROWS);
    text_rows.saturating_add(2)
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
        // Empty composer: show the dim placeholder.
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
        frame.render_widget(Paragraph::new(text), text_area);
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

/// Frames of the animated spinner shown next to the waiting label. Mirrors the
/// braille spinner used by tools like Claude Code and Codex.
const WAITING_SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Render the live activity line in the second row of the active-turn area,
/// directly above the composer. The first row remains blank to separate this
/// prompt from the transcript above. Carries the four "is it alive / how long /
/// how much / how do I stop it" signals in one glanceable line:
/// `⠹ Working… · 3s · ↑1.2k tok · esc/Ctrl+C to interrupt`. Everything it needs is
/// pure state on `TuiApp` (spinner frame, elapsed seconds, token count), so it is
/// deterministic under test and never calls the clock itself.
fn render_activity_line(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let Some(label) = app.turn_state().label() else {
        return;
    };
    let theme = app.theme;
    let accent = waiting_indicator_style(theme);
    let meta = transcript_meta_style(theme);

    let frame_index = app.spinner_frame % WAITING_SPINNER_FRAMES.len();
    let spinner = WAITING_SPINNER_FRAMES[frame_index];

    let mut spans = vec![
        Span::raw(" "),
        Span::styled(spinner, accent),
        Span::styled(format!(" {label}…"), accent),
        Span::styled(format!(" · {}s", app.elapsed_seconds), meta),
    ];
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
/// dim line at the bottom of the transcript area. Neutral/dim — never a red error
/// chip — it carries no-op guidance and success confirmations and auto-expires.
fn render_toast(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let Some(toast) = &app.toast else {
        return;
    };
    if area.height == 0 || area.width == 0 {
        return;
    }
    // A warning is slightly more prominent (yellow) than a plain note (dim), but
    // neither is ever the red reserved for real transcript errors.
    let palette = Palette::for_theme(app.theme);
    let style = match toast.severity {
        LocalDiagnosticSeverity::Warning => palette.semantic(Semantic::Warning),
        _ => palette.meta(),
    };
    // Draw on the last row of the transcript area, as a dim ephemeral note.
    let line_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
    frame.render_widget(Clear, line_area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("· {}", toast.message),
            style,
        ))),
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
    let pending = app.pending_approval_count();
    let constraints = if pending > 0 || area.width < 60 {
        [
            Constraint::Percentage(44),
            Constraint::Length(1),
            Constraint::Percentage(55),
        ]
    } else {
        [
            Constraint::Percentage(58),
            Constraint::Length(1),
            Constraint::Percentage(41),
        ]
    };
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);
    // Persistent footer: provider/model · reasoning effort · connection ·
    // permission mode on the left; context tokens · workspace on the right. The
    // connection segment is
    // always rendered through the normalized `ConnectionState` vocabulary so a
    // raw runtime status code or `{:?}` Debug string can never reach the user.
    let connection = ConnectionState::parse(&app.status.connection);
    // Both session controls stay visible: Tab cycles permission mode and
    // Shift+Tab cycles the reasoning effort applied to subsequent turns. The
    // connection label is included only when the footer is wide enough.
    let left = if pending > 0 {
        format!(
            "{} / {} · {} · {}",
            app.status.provider,
            app.status.model,
            app.status.reasoning_effort.label(),
            app.permission_mode
        )
    } else if area.width >= 110 {
        format!(
            "{} / {} · effort {} · {} · {}",
            app.status.provider,
            app.status.model,
            app.status.reasoning_effort.label(),
            connection,
            app.permission_mode
        )
    } else {
        format!(
            "{} / {} · effort {} · {}",
            app.status.provider,
            app.status.model,
            app.status.reasoning_effort.label(),
            app.permission_mode
        )
    };
    // Right: pending-approval indicator (ALWAYS shown when a request is pending,
    // regardless of active panel or terminal width — an approval is never
    // armed-but-invisible) · context tokens · workspace.
    let right = if pending > 0 {
        format!(
            "⚠ {} approval{} pending · ctx {}  {}",
            pending,
            if pending == 1 { "" } else { "s" },
            app.status.context.render(),
            app.status.workspace
        )
    } else {
        format!(
            "ctx {}  {}",
            app.status.context.render(),
            app.status.workspace
        )
    };
    frame.render_widget(
        Paragraph::new(left).style(theme_status_style(app.theme)),
        horizontal[0],
    );
    let right_style = if pending > 0 {
        approval_pending_style(app.theme)
    } else {
        theme_status_style(app.theme)
    };
    frame.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Right)
            .style(right_style),
        horizontal[2],
    );
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

/// Render the approval modal as a centered overlay (`Clear` + bordered block).
/// It shows the concrete action (reusing the real diff renderer for file edits
/// and the request's command/resource otherwise — never a paraphrase), the three
/// explicit choices with Allow once selected by default, and — when the user
/// chose "Reject & redirect" — the one-line reason input.
fn render_approval_modal(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let Some(request) = app.approval_panel.focused_request() else {
        return;
    };
    let theme = app.theme;

    let pending = app.approval_panel.pending.len();
    let focus = app.approval_panel.focused_index.unwrap_or(0) + 1;
    let title = if pending > 1 {
        format!(" Approval {focus}/{pending} ")
    } else {
        " Approval required ".to_string()
    };

    let mut lines: Vec<Line<'static>> = Vec::new();
    // Concrete action header — keep it neutral so the Allow/Reject options own the
    // green/orange signal, not the chrome.
    let header_style = theme_status_style(theme).add_modifier(Modifier::BOLD);
    lines.push(Line::from(Span::styled(
        format!("{} {}", request.action, request.resource_kind),
        header_style,
    )));
    lines.push(Line::from(Span::raw(request.resource.clone())));
    if !request.reason.is_empty() {
        lines.push(Line::from(Span::styled(
            request.reason.clone(),
            transcript_meta_style(theme),
        )));
    }
    lines.push(Line::from(Span::raw("")));

    // The concrete action body: the real diff for file edits, else the preview.
    if let Some(diff) = app
        .diff_panel
        .as_ref()
        .filter(|_| request.preview.kind == "file_edit")
    {
        for line in diff.render_text().lines() {
            lines.push(Line::from(Span::raw(line.to_string())));
        }
    } else {
        for line in request.preview.summary.lines() {
            lines.push(Line::from(Span::raw(line.to_string())));
        }
    }
    if request.preview.truncated {
        lines.push(Line::from(Span::styled(
            "… (truncated)",
            transcript_meta_style(theme),
        )));
    }
    if request.preview.redacted {
        lines.push(Line::from(Span::styled(
            "(redacted)",
            transcript_meta_style(theme),
        )));
    }

    lines.push(Line::from(Span::raw("")));

    // Border color follows the currently highlighted option so the whole modal
    // reads green for Allow and orange-red for Reject.
    let border = approval_option_style(app.approval_selection);

    if let Some(reject) = &app.reject_reason {
        // Reason capture mode: show the input and its controls.
        lines.push(Line::from(Span::styled(
            "Reject & redirect — type a reason, Enter to send, Esc to cancel:",
            border,
        )));
        lines.push(Line::from(Span::raw(format!("❯ {}", reject.text))));
    } else {
        // The three explicit choices. The highlighted row is the current
        // selection (moved with Up/Down, confirmed with Enter); the number keys
        // still trigger each directly. The default highlight is Allow once.
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
            // Allow options use deep green; non-allow (Reject) uses orange-red.
            // Selected rows reverse against that base so the highlight still
            // reads as green/red rather than a magenta accent.
            let base = approval_option_style(option);
            let style = if selected {
                base.add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                base
            };
            lines.push(Line::from(Span::styled(format!("{marker}{label}"), style)));
        }
        lines.push(Line::from(Span::styled(
            "↑/↓ select · Enter confirm · Tab next request · Esc dismiss (stays pending)",
            transcript_meta_style(theme),
        )));
    }

    // Auto-fit the modal height to the content (plus borders), capped to the
    // available area so a long preview can't overflow the screen. This replaces
    // the old fixed 86%-height box that left a large empty gap under short
    // requests.
    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let max_height = area.height.saturating_sub(2).max(3);
    let modal_height = content_height.saturating_add(2).min(max_height);
    let modal_area = centered_rect_with_height(80, modal_height, area);
    frame.render_widget(Clear, modal_clear_area(modal_area, area));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title_style(border)
        .title(title);
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the model-switch modal as a centered overlay, mirroring the approval
/// modal (`Clear` + bordered block). Each row is one provider and its default
/// model; the highlighted row is the current selection (moved with Up/Down,
/// confirmed with Enter). The live selection is marked so the user can see what
/// switching away from. Disabled providers are dimmed and labelled.
/// Base color for an approval choice row. Allow options are deep green; Reject
/// (and any non-allow choice) is orange-red so the decision reads at a glance
/// even before Enter.
fn approval_option_style(option: ApprovalOption) -> Style {
    match option {
        ApprovalOption::AllowOnce | ApprovalOption::AllowAlways => Style::default()
            .fg(Color::Rgb(34, 197, 94))
            .add_modifier(Modifier::BOLD),
        ApprovalOption::Reject => Style::default()
            .fg(Color::Rgb(249, 115, 22))
            .add_modifier(Modifier::BOLD),
    }
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
        let style = if selected {
            accent.add_modifier(Modifier::BOLD | Modifier::REVERSED)
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
        .border_style(accent)
        .title_style(accent)
        .title(" Theme ");
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
        let style = if selected {
            accent.add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else if !entry.enabled {
            transcript_meta_style(theme)
        } else {
            theme_status_style(theme)
        };
        lines.push(Line::from(Span::styled(label, style)));
    }

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
        .border_style(accent)
        .title_style(accent)
        .title(" Model ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the session picker as a centered overlay (same shape as the approval
/// and model modals). It lists the known sessions plus a synthetic "new session"
/// row; the highlighted row is confirmed with Enter, `n` starts a fresh session,
/// and Esc dismisses without switching.
fn render_session_modal(frame: &mut Frame<'_>, app: &TuiApp, area: Rect) {
    let panel = &app.session_panel;
    let theme = app.theme;
    let accent = modal_style(theme, Semantic::SessionModal);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled("Switch session", accent)));
    lines.push(Line::from(Span::raw("")));

    for (index, session) in panel.sessions.iter().enumerate() {
        let selected = panel.focused_index == Some(index);
        let marker = if selected { "❯ " } else { "  " };
        let label = format!(
            "{marker}{} · {} · {}/{}",
            session.status, session.title, session.provider_id, session.model_id
        );
        let style = if selected {
            accent.add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            theme_status_style(theme)
        };
        lines.push(Line::from(Span::styled(label, style)));
    }

    // The synthetic "new session" row sits one past the last session.
    let new_selected = panel.focused_index == Some(panel.sessions.len());
    let new_marker = if new_selected { "❯ " } else { "  " };
    let new_style = if new_selected {
        accent.add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        theme_status_style(theme)
    };
    lines.push(Line::from(Span::styled(
        format!("{new_marker}+ New session"),
        new_style,
    )));

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "↑/↓ select · Enter switch · n new · Esc cancel",
        transcript_meta_style(theme),
    )));

    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let max_height = area.height.saturating_sub(2).max(3);
    let modal_height = content_height.saturating_add(2).min(max_height);
    let modal_area = centered_rect_with_height(80, modal_height, area);
    frame.render_widget(Clear, modal_clear_area(modal_area, area));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(accent)
        .title_style(accent)
        .title(" Sessions ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

fn theme_welcome_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).welcome()
}

fn provider_health_summary(app: &TuiApp) -> String {
    if let Some(selected) = app
        .status_panel
        .provider_support
        .iter()
        .find(|provider| provider.provider_id == app.status.provider)
    {
        if selected.enabled && selected.compatible {
            return "ready".to_string();
        }
        if selected.enabled && !selected.missing.is_empty() {
            return format!("needs {}", selected.missing.join(","));
        }
        if !selected.enabled {
            return "disabled".to_string();
        }
    }

    match ConnectionState::parse(&app.status.connection) {
        ConnectionState::Failed => "failed".to_string(),
        ConnectionState::Degraded => "needs attention".to_string(),
        ConnectionState::Streaming => "streaming".to_string(),
        ConnectionState::Idle | ConnectionState::Disconnected => "ready".to_string(),
    }
}

fn extension_health_summary(app: &TuiApp) -> String {
    let Some(snapshot) = &app.extension_panel.snapshot else {
        return "plugin and MCP health pending".to_string();
    };
    if snapshot.extensions.is_empty() {
        return "no plugins installed · MCP idle".to_string();
    }
    let enabled = snapshot
        .extensions
        .iter()
        .filter(|extension| extension.enabled)
        .count();
    let healthy = snapshot
        .extensions
        .iter()
        .filter(|extension| extension.health == "healthy")
        .count();
    let diagnostics = snapshot.diagnostics.len();
    format!(
        "{} installed · {} enabled · {} healthy · {} diagnostics",
        snapshot.extensions.len(),
        enabled,
        healthy,
        diagnostics
    )
}

fn theme_block(title: &str, theme: TerminalTheme) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme_border_style(theme))
}

fn theme_border_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).border()
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

fn slash_selected_style(theme: TerminalTheme) -> Style {
    match theme {
        TerminalTheme::System | TerminalTheme::Light => Style::default()
            .fg(Color::Black)
            .bg(Color::Gray)
            .add_modifier(Modifier::BOLD),
        TerminalTheme::Dark => Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        TerminalTheme::HighContrast => Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    }
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

/// Plain-text rows currently painted in the transcript content area.
/// Must stay identical to [`render_transcript`]'s wrap → pad → scroll pipeline
/// so in-app selection copies what the user sees.
#[must_use]
pub(crate) fn visible_transcript_plain_lines_for_selection(
    app: &TuiApp,
    content_width: u16,
    content_height: u16,
) -> Vec<String> {
    if app.transcript.items.is_empty() || content_width == 0 || content_height == 0 {
        return Vec::new();
    }
    let lines = transcript_styled_lines(&app.transcript.items, app.theme);
    let wrapped = wrap_transcript_lines(lines, content_width);
    let padded = pad_block_backgrounds(wrapped, content_width);
    let visible = transcript_visible_lines(&padded, content_height, app.transcript.scroll_offset);
    let mut out: Vec<String> = visible
        .into_iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect();
    let target = usize::from(content_height);
    while out.len() < target {
        out.push(String::new());
    }
    out
}

pub fn sticky_chrome_height(app: &crate::TuiApp) -> u16 {
    let mut height: u16 = 4;
    if app.approval_modal_visible() {
        height = height.saturating_add(6);
    }
    height
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
