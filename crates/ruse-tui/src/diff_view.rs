//! VSCode-inspired edit diff rendering for tool results.
//!
//! After `Wrote` / `Edited`, the transcript expands a unified diff. This module
//! turns that text into either:
//! - **Side-by-side** columns when the content width is comfortable
//! - **Unified** (stacked) rows when the terminal is narrow
//!
//! Styling follows Codex/Claude Code: muted file headers, bold hunk markers,
//! full-line red/green backgrounds (GitHub/VSCode-like) rather than foreground-only.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::{Palette, TerminalTheme};

/// Prefer side-by-side when the available content width is at least this many
/// columns. Below that, unified is easier to scan (each side would be too thin).
pub const SIDE_BY_SIDE_MIN_WIDTH: usize = 100;

/// Leading indent under the tool status row (`  ⎿ ` / `    `).
const BODY_INDENT: &str = "    ";
const FIRST_INDENT: &str = "  ⎿ ";
/// Keep the diff tint away from the transcript's reserved right-side breathing room.
/// The left side already reserves the same four columns through the connector/indent.
const DIFF_RIGHT_INSET: usize = 4;
/// Tabs are expanded before rendering because terminal tab cells do not reliably
/// inherit span background colors across backends.
const TAB_STOP: usize = 4;

/// How a single source line participates in the diff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffLineKind {
    /// `---` / `+++` file headers.
    FileHeader,
    /// `@@` hunk header.
    HunkHeader,
    Context,
    Added,
    Removed,
}

/// One logical line from a parsed unified diff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_no: Option<u32>,
    pub new_no: Option<u32>,
    /// Content without the leading `+` / `-` / ` ` marker (headers keep full text).
    pub text: String,
}

/// Layout mode chosen from available width.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffLayout {
    Unified,
    SideBySide,
}

impl DiffLayout {
    #[must_use]
    pub fn choose(content_width: usize) -> Self {
        if content_width >= SIDE_BY_SIDE_MIN_WIDTH {
            Self::SideBySide
        } else {
            Self::Unified
        }
    }
}

/// Parse a unified-diff body into numbered lines.
///
/// Accepts full patches (`---/+++/@@`) and bare `+/-` dumps from edit tools.
#[must_use]
pub fn parse_unified_diff(detail: &str) -> Vec<DiffLine> {
    let mut out = Vec::new();
    let mut old_no: u32 = 0;
    let mut new_no: u32 = 0;
    let mut tracking = false;

    for raw in detail.lines() {
        if raw.starts_with("--- ") || raw.starts_with("+++ ") {
            out.push(DiffLine {
                kind: DiffLineKind::FileHeader,
                old_no: None,
                new_no: None,
                text: raw.to_string(),
            });
            continue;
        }
        if raw.starts_with("@@") {
            if let Some((o, n)) = parse_hunk_header(raw) {
                old_no = o;
                new_no = n;
                tracking = true;
            }
            out.push(DiffLine {
                kind: DiffLineKind::HunkHeader,
                old_no: None,
                new_no: None,
                text: raw.to_string(),
            });
            continue;
        }

        if raw.starts_with('+') && !raw.starts_with("+++") {
            let text = raw[1..].to_string();
            let line_no = if tracking {
                let n = new_no;
                new_no = new_no.saturating_add(1);
                Some(n.max(1))
            } else {
                None
            };
            out.push(DiffLine {
                kind: DiffLineKind::Added,
                old_no: None,
                new_no: line_no,
                text,
            });
            continue;
        }
        if raw.starts_with('-') && !raw.starts_with("---") {
            let text = raw[1..].to_string();
            let line_no = if tracking {
                let n = old_no;
                old_no = old_no.saturating_add(1);
                Some(n.max(1))
            } else {
                None
            };
            out.push(DiffLine {
                kind: DiffLineKind::Removed,
                old_no: line_no,
                new_no: None,
                text,
            });
            continue;
        }

        // Context (leading space) or unmarked continuation.
        let text = if let Some(stripped) = raw.strip_prefix(' ') {
            stripped.to_string()
        } else {
            raw.to_string()
        };
        let (o, n) = if tracking {
            let o = old_no;
            let n = new_no;
            old_no = old_no.saturating_add(1);
            new_no = new_no.saturating_add(1);
            (Some(o.max(1)), Some(n.max(1)))
        } else {
            (None, None)
        };
        out.push(DiffLine {
            kind: DiffLineKind::Context,
            old_no: o,
            new_no: n,
            text,
        });
    }
    out
}

fn parse_hunk_header(header: &str) -> Option<(u32, u32)> {
    // @@ -old_start,count +new_start,count @@
    // @@ -old_start +new_start @@
    let mut old_start = None;
    let mut new_start = None;
    for token in header.split_whitespace() {
        if let Some(rest) = token.strip_prefix('-') {
            let start = rest.split(',').next()?;
            old_start = start.parse().ok();
        } else if let Some(rest) = token.strip_prefix('+') {
            let start = rest.split(',').next()?;
            new_start = start.parse().ok();
        }
    }
    Some((old_start.unwrap_or(1), new_start.unwrap_or(1)))
}

/// Render a unified-diff body for a finished edit/write tool result.
#[must_use]
pub fn render_edit_diff_lines(
    detail: &str,
    content_width: usize,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    let parsed = parse_unified_diff(detail);
    if parsed.is_empty() {
        return Vec::new();
    }
    match DiffLayout::choose(content_width) {
        DiffLayout::SideBySide => render_side_by_side(&parsed, content_width, theme),
        DiffLayout::Unified => render_unified(&parsed, content_width, theme),
    }
}

fn render_unified(
    lines: &[DiffLine],
    content_width: usize,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    let gutter = gutter_width(lines);
    let block_width = diff_block_width(content_width);
    let meta = Palette::for_theme(theme).meta();
    let mut out = Vec::with_capacity(lines.len());
    let mut first_output_row = true;

    for line in lines {
        for row in render_unified_content_rows(line, gutter, block_width, theme) {
            let indent = if first_output_row {
                FIRST_INDENT
            } else {
                BODY_INDENT
            };
            let mut spans = vec![Span::styled(indent.to_string(), meta)];
            spans.extend(row.spans);
            out.push(Line::from(spans));
            first_output_row = false;
        }
    }
    out
}

fn render_unified_content_rows(
    line: &DiffLine,
    gutter: usize,
    block_width: usize,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    let meta = Palette::for_theme(theme).meta();
    match line.kind {
        DiffLineKind::FileHeader | DiffLineKind::HunkHeader => {
            let marker = if line.kind == DiffLineKind::HunkHeader {
                meta.add_modifier(Modifier::BOLD)
            } else {
                meta
            };
            display_chunks(&line.text, block_width)
                .into_iter()
                .map(|text| Line::from(Span::styled(text, marker)))
                .collect()
        }
        DiffLineKind::Context | DiffLineKind::Added | DiffLineKind::Removed => {
            let sign = match line.kind {
                DiffLineKind::Added => "+",
                DiffLineKind::Removed => "-",
                _ => " ",
            };
            let line_no = match line.kind {
                DiffLineKind::Added => line.new_no,
                DiffLineKind::Removed => line.old_no,
                DiffLineKind::Context => line.old_no.or(line.new_no),
                _ => None,
            };
            let number = format_line_no(line_no, gutter);
            let prefix = format!("{number} {sign} ");
            let prefix_width = prefix.width().min(block_width);
            let text_width = block_width.saturating_sub(prefix_width);
            let content_style = content_style_for(line.kind, theme);

            display_chunks(&line.text, text_width)
                .into_iter()
                .enumerate()
                .map(|(index, text)| {
                    let row_prefix = if index == 0 {
                        prefix.clone()
                    } else {
                        " ".repeat(prefix_width)
                    };
                    let cell = truncate_pad(&format!("{row_prefix}{text}"), block_width);
                    // Keep the tint on the explicit diff cell only. A line-level
                    // background would also color the left indent and trigger the
                    // transcript's full-width message padding on the right.
                    Line::from(Span::styled(cell, content_style))
                })
                .collect()
        }
    }
}

/// Align remove/add groups into side-by-side rows, then render.
fn render_side_by_side(
    lines: &[DiffLine],
    content_width: usize,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    let pairs = align_side_by_side(lines);
    let available = diff_block_width(content_width);
    // left | divider | right
    let divider = " │ ";
    let half = available.saturating_sub(divider.width()) / 2;
    let left_w = half.max(12);
    let right_w = available.saturating_sub(left_w + divider.width()).max(12);
    let gutter = gutter_width(lines);

    let mut out = Vec::with_capacity(pairs.len());
    for (index, pair) in pairs.iter().enumerate() {
        let indent = if index == 0 {
            FIRST_INDENT
        } else {
            BODY_INDENT
        };
        out.push(render_side_by_side_row(
            pair, indent, left_w, right_w, gutter, theme,
        ));
    }
    out
}

#[derive(Clone, Debug)]
enum SidePair {
    Header(DiffLine),
    /// Left = old side, right = new side. `None` means empty padding cell.
    Row {
        left: Option<DiffLine>,
        right: Option<DiffLine>,
    },
}

fn align_side_by_side(lines: &[DiffLine]) -> Vec<SidePair> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        match lines[i].kind {
            DiffLineKind::FileHeader | DiffLineKind::HunkHeader => {
                out.push(SidePair::Header(lines[i].clone()));
                i += 1;
            }
            DiffLineKind::Context => {
                out.push(SidePair::Row {
                    left: Some(lines[i].clone()),
                    right: Some(lines[i].clone()),
                });
                i += 1;
            }
            DiffLineKind::Removed | DiffLineKind::Added => {
                let mut removed = Vec::new();
                let mut added = Vec::new();
                while i < lines.len() && lines[i].kind == DiffLineKind::Removed {
                    removed.push(lines[i].clone());
                    i += 1;
                }
                while i < lines.len() && lines[i].kind == DiffLineKind::Added {
                    added.push(lines[i].clone());
                    i += 1;
                }
                let n = removed.len().max(added.len());
                for k in 0..n {
                    out.push(SidePair::Row {
                        left: removed.get(k).cloned(),
                        right: added.get(k).cloned(),
                    });
                }
            }
        }
    }
    out
}

fn render_side_by_side_row(
    pair: &SidePair,
    indent: &str,
    left_w: usize,
    right_w: usize,
    gutter: usize,
    theme: TerminalTheme,
) -> Line<'static> {
    let meta = Palette::for_theme(theme).meta();
    match pair {
        SidePair::Header(line) => {
            let style = if line.kind == DiffLineKind::HunkHeader {
                meta.add_modifier(Modifier::BOLD)
            } else {
                meta
            };
            let block_width = left_w + " │ ".width() + right_w;
            Line::from(vec![
                Span::styled(indent.to_string(), meta),
                Span::styled(truncate_pad(&line.text, block_width), style),
            ])
        }
        SidePair::Row { left, right } => {
            let left_spans = side_cell(left.as_ref(), left_w, gutter, true, theme);
            let right_spans = side_cell(right.as_ref(), right_w, gutter, false, theme);
            let mut spans = vec![Span::styled(indent.to_string(), meta)];
            spans.extend(left_spans);
            spans.push(Span::styled(" │ ".to_string(), meta));
            spans.extend(right_spans);
            // Prefer a neutral line style; cell backgrounds live on the spans.
            Line::from(spans)
        }
    }
}

fn side_cell(
    line: Option<&DiffLine>,
    width: usize,
    gutter: usize,
    is_left: bool,
    theme: TerminalTheme,
) -> Vec<Span<'static>> {
    let Some(line) = line else {
        return vec![Span::raw(" ".repeat(width))];
    };
    // On the left (old) side, only show removals + context. On the right, only
    // additions + context. Headers never reach here.
    let display_kind = match (is_left, line.kind) {
        (true, DiffLineKind::Added) => {
            return vec![Span::raw(" ".repeat(width))];
        }
        (false, DiffLineKind::Removed) => {
            return vec![Span::raw(" ".repeat(width))];
        }
        (_, kind) => kind,
    };

    let sign = match display_kind {
        DiffLineKind::Added => "+",
        DiffLineKind::Removed => "-",
        _ => " ",
    };
    let line_no = if is_left {
        line.old_no.or(line.new_no)
    } else {
        line.new_no.or(line.old_no)
    };
    let number = format_line_no(line_no, gutter);
    let prefix = format!("{number} {sign} ");
    let prefix_w = prefix.width();
    let text_w = width.saturating_sub(prefix_w);
    let text = truncate_pad(&line.text, text_w);
    let style = content_style_for(display_kind, theme);
    vec![Span::styled(prefix, style), Span::styled(text, style)]
}

fn gutter_width(lines: &[DiffLine]) -> usize {
    let max = lines
        .iter()
        .filter_map(|l| l.old_no.max(l.new_no))
        .max()
        .unwrap_or(0);
    max.to_string().len().max(1)
}

fn format_line_no(no: Option<u32>, gutter: usize) -> String {
    match no {
        Some(n) => format!("{n:>gutter$}"),
        None => " ".repeat(gutter),
    }
}

fn diff_block_width(content_width: usize) -> usize {
    content_width
        .saturating_sub(FIRST_INDENT.width() + DIFF_RIGHT_INSET)
        .max(1)
}

fn display_chunks(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let expanded = expand_tabs(text);
    if expanded.is_empty() {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut used = 0usize;
    for ch in expanded.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used > 0 && used + char_width > width {
            chunks.push(std::mem::take(&mut current));
            used = 0;
        }
        if char_width > width && current.is_empty() {
            continue;
        }
        current.push(ch);
        used += char_width;
    }
    if !current.is_empty() || chunks.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn expand_tabs(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut column = 0usize;
    for ch in text.chars() {
        if ch == '\t' {
            let spaces = TAB_STOP - (column % TAB_STOP);
            out.push_str(&" ".repeat(spaces));
            column += spaces;
        } else {
            out.push(ch);
            column += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        }
    }
    out
}

fn truncate_pad(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let expanded = expand_tabs(text);
    let mut out = String::new();
    let mut used = 0usize;
    for ch in expanded.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    if used < width {
        out.push_str(&" ".repeat(width - used));
    }
    out
}

fn change_bg(kind: DiffLineKind, theme: TerminalTheme) -> Option<Color> {
    let palette = Palette::for_theme(theme);
    match kind {
        DiffLineKind::Added => palette.diff_added_bg(),
        DiffLineKind::Removed => palette.diff_removed_bg(),
        _ => None,
    }
}

fn content_style_for(kind: DiffLineKind, theme: TerminalTheme) -> Style {
    let palette = Palette::for_theme(theme);
    let mut style = match kind {
        DiffLineKind::Added => palette.diff_added(),
        DiffLineKind::Removed => palette.diff_removed(),
        _ => return palette.meta(),
    };
    if let Some(bg) = change_bg(kind, theme) {
        style = style.bg(bg);
    }
    style
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;

    #[test]
    fn parse_assigns_line_numbers_from_hunk() {
        let lines =
            parse_unified_diff("--- a/f\n+++ b/f\n@@ -10,2 +10,2 @@\n context\n-old\n+new\n");
        assert!(
            lines
                .iter()
                .any(|l| l.kind == DiffLineKind::Removed && l.old_no == Some(11))
        );
        assert!(
            lines
                .iter()
                .any(|l| l.kind == DiffLineKind::Added && l.new_no == Some(11))
        );
    }

    #[test]
    fn layout_threshold() {
        assert_eq!(DiffLayout::choose(99), DiffLayout::Unified);
        assert_eq!(DiffLayout::choose(100), DiffLayout::SideBySide);
    }

    #[test]
    fn side_by_side_pairs_remove_add() {
        let lines = parse_unified_diff("@@ -1 +1 @@\n-old\n+new\n");
        let pairs = align_side_by_side(&lines);
        assert!(matches!(
            &pairs[1],
            SidePair::Row {
                left: Some(l),
                right: Some(r)
            } if l.kind == DiffLineKind::Removed && r.kind == DiffLineKind::Added
        ));
    }

    #[test]
    fn render_contains_old_and_new_markers() {
        let rendered = render_edit_diff_lines(
            "--- src/main.rs\n+++ src/main.rs\n@@ -1 +1 @@\n-old line\n+new line\n",
            80,
            TerminalTheme::Dark,
        );
        let text: String = rendered
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("old line"), "{text}");
        assert!(text.contains("new line"), "{text}");
        assert!(text.contains('-') || text.contains('+'), "{text}");
    }

    #[test]
    fn wide_layout_emits_divider() {
        let rendered = render_edit_diff_lines(
            "@@ -1 +1 @@\n-old line\n+new line\n",
            120,
            TerminalTheme::Dark,
        );
        let text: String = rendered
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<_>>()
            .join("");
        assert!(
            text.contains('│'),
            "side-by-side should draw a column divider: {text}"
        );
    }

    #[test]
    fn unified_change_background_stays_inside_insets_and_covers_whitespace() {
        let content_width = 80;
        let rendered = render_edit_diff_lines(
            "@@ -1 +1 @@\n-\told \t\n+\tnew \t\n",
            content_width,
            TerminalTheme::Dark,
        );
        let removed = rendered
            .iter()
            .find(|line| line.spans.iter().any(|span| span.content.contains("old")))
            .expect("removed row");

        assert!(
            removed.style.bg.is_none(),
            "line background must stay neutral"
        );
        assert_eq!(removed.width(), content_width - DIFF_RIGHT_INSET);
        assert_eq!(removed.spans[0].style.bg, None, "left indent stays neutral");
        assert_eq!(
            removed.spans[1].style.bg,
            Palette::for_theme(TerminalTheme::Dark).diff_removed_bg()
        );
        assert_eq!(removed.spans[1].width(), diff_block_width(content_width));
        assert!(!removed.spans[1].content.contains('\t'));
        assert!(removed.spans[1].content.ends_with(' '));

        // Verify the final terminal cells, not only the intermediate spans:
        // left/right insets stay neutral while trailing whitespace inside the
        // change cell receives the same background as visible characters.
        let removed_bg = Palette::for_theme(TerminalTheme::Dark)
            .diff_removed_bg()
            .expect("removed background");
        let mut buffer = Buffer::empty(Rect::new(0, 0, content_width as u16, 3));
        let area = buffer.area;
        Paragraph::new(rendered).render(area, &mut buffer);
        for x in 0..FIRST_INDENT.width() as u16 {
            assert_eq!(buffer[(x, 1)].bg, Color::Reset, "left inset x={x}");
        }
        for x in FIRST_INDENT.width() as u16..(content_width - DIFF_RIGHT_INSET) as u16 {
            assert_eq!(buffer[(x, 1)].bg, removed_bg, "diff cell x={x}");
        }
        for x in (content_width - DIFF_RIGHT_INSET) as u16..content_width as u16 {
            assert_eq!(buffer[(x, 1)].bg, Color::Reset, "right inset x={x}");
        }
        assert_eq!(
            buffer[((content_width - DIFF_RIGHT_INSET - 1) as u16, 1)].symbol(),
            " ",
            "the final tinted cell is explicit whitespace"
        );
    }

    #[test]
    fn side_by_side_change_background_keeps_both_outer_insets_neutral() {
        let content_width = 120;
        let rendered = render_edit_diff_lines(
            "@@ -1 +1 @@\n-\told\n+\tnew\n",
            content_width,
            TerminalTheme::Dark,
        );
        let change = rendered
            .iter()
            .find(|line| line.spans.iter().any(|span| span.content.contains("old")))
            .expect("side-by-side change row");

        assert!(
            change.style.bg.is_none(),
            "line background must stay neutral"
        );
        assert_eq!(change.width(), content_width - DIFF_RIGHT_INSET);
        assert_eq!(change.spans[0].style.bg, None, "left indent stays neutral");
        assert!(
            change.spans.iter().all(|span| !span.content.contains('\t')),
            "tabs must be expanded into styled spaces"
        );
        assert_eq!(
            change.spans[1].style.bg,
            Palette::for_theme(TerminalTheme::Dark).diff_removed_bg()
        );
        assert_eq!(
            change.spans.last().expect("right cell").style.bg,
            Palette::for_theme(TerminalTheme::Dark).diff_added_bg()
        );
        assert!(
            change
                .spans
                .last()
                .expect("right cell")
                .content
                .ends_with(' ')
        );
    }
}
