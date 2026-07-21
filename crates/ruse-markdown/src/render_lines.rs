//! Render a Markdown [`Document`] into width-aware styled lines.

use unicode_width::UnicodeWidthStr;

use crate::highlight::highlight_code_lines;
use crate::ir::{Block, Document, Inline, ListItem, TableAlign, TableCell};
use crate::parse::parse_markdown_with_options;
use crate::theme::{RenderOptions, RenderedLine, SegmentRole, StyledSpan};

/// Parse and render Markdown source to styled terminal lines.
#[must_use]
pub fn render_markdown_lines(source: &str, options: &RenderOptions) -> Vec<RenderedLine> {
    let doc = parse_markdown_with_options(source, options.streaming);
    render_document_lines(&doc, options)
}

/// Render a pre-parsed document.
#[must_use]
pub fn render_document_lines(doc: &Document, options: &RenderOptions) -> Vec<RenderedLine> {
    let mut lines = Vec::new();
    let width = options.width;
    for (index, block) in doc.blocks.iter().enumerate() {
        let next_is_list = matches!(doc.blocks.get(index + 1), Some(Block::List { .. }));
        render_block(block, options, width, 0, next_is_list, &mut lines);
    }
    // Drop trailing blank lines.
    while lines
        .last()
        .is_some_and(|line| line.plain_text().trim().is_empty())
    {
        lines.pop();
    }
    lines
}

fn render_block(
    block: &Block,
    options: &RenderOptions,
    width: usize,
    depth: usize,
    suppress_trailing_blank: bool,
    out: &mut Vec<RenderedLine>,
) {
    match block {
        Block::Paragraph { inlines } => {
            let spans = render_inlines(inlines, options, SegmentRole::Plain);
            push_wrapped(out, spans, width, "");
            if !suppress_trailing_blank {
                out.push(RenderedLine::default());
            }
        }
        Block::Heading { level, inlines } => {
            let role = SegmentRole::Heading(*level);
            let mut spans = Vec::new();
            if *level >= 3 {
                spans.push(StyledSpan {
                    text: format!("{} ", "#".repeat((*level as usize).min(6))),
                    role,
                });
            }
            spans.extend(render_inlines(inlines, options, role));
            push_wrapped(out, spans, width, "");
            out.push(RenderedLine::default());
        }
        Block::CodeBlock { lang, code } => {
            let fence = format!("```{}", lang.as_deref().unwrap_or(""));
            out.push(RenderedLine::single(fence, SegmentRole::CodeBlockBorder));
            let body = highlight_code_lines(code, lang.as_deref(), options.highlight_code);
            let indent = &options.code_block_indent;
            for line in body {
                let mut spans = vec![StyledSpan {
                    text: indent.clone(),
                    role: SegmentRole::CodeBlock,
                }];
                spans.extend(line.spans);
                out.push(RenderedLine { spans });
            }
            out.push(RenderedLine::single("```", SegmentRole::CodeBlockBorder));
            if !suppress_trailing_blank {
                out.push(RenderedLine::default());
            }
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            render_list(items, *ordered, *start, options, width, depth, out);
        }
        Block::BlockQuote { blocks } => {
            let content_width = if width > 2 { width - 2 } else { width };
            let mut inner = Vec::new();
            for (i, child) in blocks.iter().enumerate() {
                let next_list = matches!(blocks.get(i + 1), Some(Block::List { .. }));
                render_block(child, options, content_width, depth, next_list, &mut inner);
            }
            while inner
                .last()
                .is_some_and(|line| line.plain_text().trim().is_empty())
            {
                inner.pop();
            }
            for line in inner {
                let mut spans = vec![StyledSpan {
                    text: "│ ".to_string(),
                    role: SegmentRole::QuoteBorder,
                }];
                for span in line.spans {
                    spans.push(StyledSpan {
                        text: span.text,
                        role: match span.role {
                            SegmentRole::Plain => SegmentRole::Quote,
                            other => other,
                        },
                    });
                }
                out.push(RenderedLine { spans });
            }
            if !suppress_trailing_blank {
                out.push(RenderedLine::default());
            }
        }
        Block::ThematicBreak => {
            let bar_width = if width == 0 { 40 } else { width.min(80) };
            out.push(RenderedLine::single(
                "─".repeat(bar_width.max(3)),
                SegmentRole::Hr,
            ));
            if !suppress_trailing_blank {
                out.push(RenderedLine::default());
            }
        }
        Block::Table {
            headers,
            alignments,
            rows,
        } => {
            render_table(headers, alignments, rows, options, width, out);
            if !suppress_trailing_blank {
                out.push(RenderedLine::default());
            }
        }
        Block::Html { raw } => {
            for line in raw.lines() {
                out.push(RenderedLine::plain(line.to_string()));
            }
            if !suppress_trailing_blank {
                out.push(RenderedLine::default());
            }
        }
    }
}

fn render_list(
    items: &[ListItem],
    ordered: bool,
    start: u64,
    options: &RenderOptions,
    width: usize,
    depth: usize,
    out: &mut Vec<RenderedLine>,
) {
    let indent = "    ".repeat(depth);
    for (index, item) in items.iter().enumerate() {
        let bullet = if ordered {
            format!("{}. ", start + index as u64)
        } else {
            "- ".to_string()
        };
        let task = match item.task {
            Some(true) => "[x] ".to_string(),
            Some(false) => "[ ] ".to_string(),
            None => String::new(),
        };
        let marker = format!("{indent}{bullet}{task}");
        let marker_width = UnicodeWidthStr::width(marker.as_str());
        let content_width = if width > marker_width {
            width - marker_width
        } else {
            width
        };
        let continuation = format!(
            "{indent}{}",
            " ".repeat(
                UnicodeWidthStr::width(bullet.as_str()) + UnicodeWidthStr::width(task.as_str()),
            ),
        );

        let mut first = true;
        for block in &item.blocks {
            match block {
                Block::List {
                    ordered,
                    start,
                    items,
                } => {
                    render_list(items, *ordered, *start, options, width, depth + 1, out);
                    first = false;
                }
                Block::Paragraph { inlines } | Block::Heading { inlines, .. } => {
                    let spans = render_inlines(inlines, options, SegmentRole::Plain);
                    let (prefix, bullet_role) = if first {
                        first = false;
                        (marker.as_str(), SegmentRole::ListBullet)
                    } else {
                        (continuation.as_str(), SegmentRole::Plain)
                    };
                    let mut head = vec![StyledSpan {
                        text: prefix.to_string(),
                        role: bullet_role,
                    }];
                    head.extend(spans);
                    push_wrapped_with_hanging(out, head, width, prefix, bullet_role);
                }
                other => {
                    let mut nested = Vec::new();
                    render_block(other, options, content_width, depth, true, &mut nested);
                    for line in nested {
                        if first {
                            first = false;
                            let mut spans = vec![StyledSpan {
                                text: marker.clone(),
                                role: SegmentRole::ListBullet,
                            }];
                            spans.extend(line.spans);
                            out.push(RenderedLine { spans });
                        } else {
                            let mut spans = vec![StyledSpan {
                                text: continuation.clone(),
                                role: SegmentRole::Plain,
                            }];
                            spans.extend(line.spans);
                            out.push(RenderedLine { spans });
                        }
                    }
                }
            }
        }
        if first {
            out.push(RenderedLine::single(marker, SegmentRole::ListBullet));
        }
    }
}

fn render_table(
    headers: &[TableCell],
    alignments: &[TableAlign],
    rows: &[Vec<TableCell>],
    options: &RenderOptions,
    width: usize,
    out: &mut Vec<RenderedLine>,
) {
    if headers.is_empty() {
        return;
    }
    let cols = headers.len();
    let mut col_widths = vec![3usize; cols];
    for (i, cell) in headers.iter().enumerate() {
        col_widths[i] = col_widths[i].max(plain_cell_width(cell));
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(cols) {
            col_widths[i] = col_widths[i].max(plain_cell_width(cell));
        }
    }

    // Shrink columns if needed.
    let border_overhead = 3 * cols + 1;
    if width > 0 {
        let available = width.saturating_sub(border_overhead);
        if available < cols {
            // Too narrow — fall back to plain text.
            let mut plain = String::new();
            for (i, cell) in headers.iter().enumerate() {
                if i > 0 {
                    plain.push_str(" | ");
                }
                plain.push_str(&inline_plain(&cell.inlines));
            }
            out.push(RenderedLine::plain(plain));
            for row in rows {
                let mut plain = String::new();
                for (i, cell) in row.iter().enumerate().take(cols) {
                    if i > 0 {
                        plain.push_str(" | ");
                    }
                    plain.push_str(&inline_plain(&cell.inlines));
                }
                out.push(RenderedLine::plain(plain));
            }
            return;
        }
        let total: usize = col_widths.iter().sum();
        if total > available {
            let mut remaining = available;
            let last = cols.saturating_sub(1);
            for (i, width) in col_widths.iter_mut().enumerate() {
                let share = if i == last {
                    remaining
                } else {
                    (*width * available / total).max(1).min(remaining)
                };
                *width = share;
                remaining = remaining.saturating_sub(share);
            }
        }
    }

    out.push(render_table_row(
        headers,
        &col_widths,
        alignments,
        options,
        true,
    ));
    let mut sep = String::from("|");
    for (i, w) in col_widths.iter().enumerate() {
        let align = alignments.get(i).copied().unwrap_or_default();
        let (l, r) = match align {
            TableAlign::Left => (':', '-'),
            TableAlign::Right => ('-', ':'),
            TableAlign::Center => (':', ':'),
            TableAlign::None => ('-', '-'),
        };
        sep.push(l);
        sep.push_str(&"-".repeat((*w).max(1)));
        sep.push(r);
        sep.push('|');
    }
    out.push(RenderedLine::single(sep, SegmentRole::TableBorder));

    for row in rows {
        let mut cells = row.to_vec();
        while cells.len() < cols {
            cells.push(TableCell {
                inlines: Vec::new(),
            });
        }
        out.push(render_table_row(
            &cells,
            &col_widths,
            alignments,
            options,
            false,
        ));
    }
}

fn render_table_row(
    cells: &[TableCell],
    col_widths: &[usize],
    alignments: &[TableAlign],
    options: &RenderOptions,
    header: bool,
) -> RenderedLine {
    let mut spans = vec![StyledSpan {
        text: "| ".to_string(),
        role: SegmentRole::TableBorder,
    }];
    for (i, width) in col_widths.iter().enumerate() {
        let cell = cells.get(i);
        let inlines = cell.map(|c| c.inlines.as_slice()).unwrap_or(&[]);
        let role = if header {
            SegmentRole::Bold
        } else {
            SegmentRole::Plain
        };
        let content_spans = render_inlines(inlines, options, role);
        let text: String = content_spans.iter().map(|s| s.text.as_str()).collect();
        let pad = width.saturating_sub(UnicodeWidthStr::width(text.as_str()));
        let align = alignments.get(i).copied().unwrap_or_default();
        let (left, right) = match align {
            TableAlign::Right => (pad, 0),
            TableAlign::Center => (pad / 2, pad - pad / 2),
            _ => (0, pad),
        };
        if left > 0 {
            spans.push(StyledSpan {
                text: " ".repeat(left),
                role: SegmentRole::Plain,
            });
        }
        spans.extend(content_spans);
        if right > 0 {
            spans.push(StyledSpan {
                text: " ".repeat(right),
                role: SegmentRole::Plain,
            });
        }
        spans.push(StyledSpan {
            text: " | ".to_string(),
            role: SegmentRole::TableBorder,
        });
    }
    // Trim trailing space from last separator.
    if let Some(last) = spans.last_mut() {
        if last.text.ends_with(' ') {
            last.text.pop();
        }
    }
    RenderedLine { spans }
}

fn plain_cell_width(cell: &TableCell) -> usize {
    UnicodeWidthStr::width(inline_plain(&cell.inlines).as_str()).max(1)
}

fn inline_plain(inlines: &[Inline]) -> String {
    let mut out = String::new();
    flatten_plain(inlines, &mut out);
    out
}

fn flatten_plain(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(text) | Inline::Code(text) | Inline::Html(text) => out.push_str(text),
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            Inline::Strong(inner) | Inline::Emphasis(inner) | Inline::Strikethrough(inner) => {
                flatten_plain(inner, out);
            }
            Inline::Link { text, .. } => flatten_plain(text, out),
            Inline::Image { alt, .. } => out.push_str(alt),
        }
    }
}

fn render_inlines(
    inlines: &[Inline],
    options: &RenderOptions,
    base: SegmentRole,
) -> Vec<StyledSpan> {
    let mut spans = Vec::new();
    for inline in inlines {
        match inline {
            Inline::Text(text) | Inline::Html(text) => spans.push(StyledSpan {
                text: text.clone(),
                role: base,
            }),
            Inline::SoftBreak => spans.push(StyledSpan {
                text: " ".to_string(),
                role: base,
            }),
            Inline::HardBreak => spans.push(StyledSpan {
                text: "\n".to_string(),
                role: base,
            }),
            Inline::Code(code) => spans.push(StyledSpan {
                text: code.clone(),
                role: SegmentRole::Code,
            }),
            Inline::Strong(inner) => {
                spans.extend(render_inlines(inner, options, SegmentRole::Bold));
            }
            Inline::Emphasis(inner) => {
                spans.extend(render_inlines(inner, options, SegmentRole::Italic));
            }
            Inline::Strikethrough(inner) => {
                spans.extend(render_inlines(inner, options, SegmentRole::Strikethrough));
            }
            Inline::Link { text, dest, .. } => {
                let label = render_inlines(text, options, SegmentRole::Link);
                let label_text: String = label.iter().map(|s| s.text.as_str()).collect();
                if options.osc8_links {
                    let open = format!("\x1b]8;;{dest}\x1b\\");
                    let close = "\x1b]8;;\x1b\\";
                    spans.push(StyledSpan {
                        text: format!("{open}{label_text}{close}"),
                        role: SegmentRole::Link,
                    });
                } else if label_text == *dest
                    || dest.strip_prefix("mailto:") == Some(label_text.as_str())
                {
                    spans.push(StyledSpan {
                        text: label_text,
                        role: SegmentRole::Link,
                    });
                } else {
                    spans.push(StyledSpan {
                        text: label_text,
                        role: SegmentRole::Link,
                    });
                    spans.push(StyledSpan {
                        text: format!(" ({dest})"),
                        role: SegmentRole::LinkUrl,
                    });
                }
            }
            Inline::Image { alt, dest } => {
                spans.push(StyledSpan {
                    text: if alt.is_empty() {
                        dest.clone()
                    } else {
                        format!("{alt} ({dest})")
                    },
                    role: SegmentRole::LinkUrl,
                });
            }
        }
    }
    spans
}

fn push_wrapped(out: &mut Vec<RenderedLine>, spans: Vec<StyledSpan>, width: usize, prefix: &str) {
    push_wrapped_with_hanging(out, spans, width, prefix, SegmentRole::Plain);
}

fn push_wrapped_with_hanging(
    out: &mut Vec<RenderedLine>,
    spans: Vec<StyledSpan>,
    width: usize,
    prefix: &str,
    prefix_role: SegmentRole,
) {
    // Expand hard breaks first.
    let mut segments: Vec<Vec<StyledSpan>> = vec![Vec::new()];
    for span in spans {
        if span.text.contains('\n') {
            let parts: Vec<&str> = span.text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    segments.push(Vec::new());
                }
                if !part.is_empty() {
                    segments.last_mut().expect("segment").push(StyledSpan {
                        text: (*part).to_string(),
                        role: span.role,
                    });
                }
            }
        } else {
            segments.last_mut().expect("segment").push(span);
        }
    }

    for (seg_index, segment) in segments.into_iter().enumerate() {
        if width == 0 {
            let mut line_spans = Vec::new();
            if seg_index == 0 && !prefix.is_empty() {
                // prefix already included in spans for list case
            }
            line_spans.extend(segment);
            out.push(RenderedLine { spans: line_spans });
            continue;
        }

        let mut lines = wrap_spans(segment, width);
        if lines.is_empty() {
            lines.push(RenderedLine::default());
        }
        // For list items, first line already contains the marker in spans.
        // Continuation lines need hanging indent.
        if !prefix.is_empty() {
            for (i, line) in lines.iter_mut().enumerate() {
                if i == 0 {
                    continue;
                }
                let mut spans = vec![StyledSpan {
                    text: prefix.to_string(),
                    role: prefix_role,
                }];
                // For hanging indent, use spaces equal to prefix width.
                spans[0].text = " ".repeat(UnicodeWidthStr::width(prefix));
                spans[0].role = SegmentRole::Plain;
                spans.extend(std::mem::take(&mut line.spans));
                line.spans = spans;
            }
        }
        out.extend(lines);
    }
}

fn wrap_spans(spans: Vec<StyledSpan>, width: usize) -> Vec<RenderedLine> {
    if width == 0 {
        return vec![RenderedLine { spans }];
    }
    let mut lines = Vec::new();
    let mut current: Vec<StyledSpan> = Vec::new();
    let mut current_width = 0usize;

    for span in spans {
        // Split span text into words while preserving spaces as break opportunities.
        let mut buf = String::new();
        let flush_buf = |buf: &mut String,
                         role: SegmentRole,
                         current: &mut Vec<StyledSpan>,
                         current_width: &mut usize,
                         lines: &mut Vec<RenderedLine>| {
            if buf.is_empty() {
                return;
            }
            let piece = std::mem::take(buf);
            let piece_width = UnicodeWidthStr::width(piece.as_str());
            if *current_width > 0 && *current_width + piece_width > width {
                lines.push(RenderedLine {
                    spans: std::mem::take(current),
                });
                *current_width = 0;
            }
            // Hard-split overlong tokens.
            if piece_width > width {
                let mut rest = piece;
                while UnicodeWidthStr::width(rest.as_str()) > width {
                    let mut acc = String::new();
                    let mut acc_w = 0usize;
                    let mut cut = 0usize;
                    for (idx, ch) in rest.char_indices() {
                        let w = UnicodeWidthStr::width(ch.to_string().as_str());
                        if acc_w + w > width && !acc.is_empty() {
                            break;
                        }
                        acc.push(ch);
                        acc_w += w;
                        cut = idx + ch.len_utf8();
                    }
                    current.push(StyledSpan { text: acc, role });
                    lines.push(RenderedLine {
                        spans: std::mem::take(current),
                    });
                    *current_width = 0;
                    rest = rest[cut..].to_string();
                }
                if !rest.is_empty() {
                    *current_width = UnicodeWidthStr::width(rest.as_str());
                    current.push(StyledSpan { text: rest, role });
                }
            } else {
                current.push(StyledSpan { text: piece, role });
                *current_width += piece_width;
            }
        };

        for ch in span.text.chars() {
            if ch.is_whitespace() && ch != '\n' {
                flush_buf(
                    &mut buf,
                    span.role,
                    &mut current,
                    &mut current_width,
                    &mut lines,
                );
                let space = ch.to_string();
                let w = UnicodeWidthStr::width(space.as_str());
                if current_width > 0 && current_width + w > width {
                    lines.push(RenderedLine {
                        spans: std::mem::take(&mut current),
                    });
                    current_width = 0;
                    // Skip leading spaces on new lines.
                    continue;
                }
                if current_width == 0 {
                    continue;
                }
                current.push(StyledSpan {
                    text: space,
                    role: span.role,
                });
                current_width += w;
            } else {
                buf.push(ch);
            }
        }
        flush_buf(
            &mut buf,
            span.role,
            &mut current,
            &mut current_width,
            &mut lines,
        );
    }

    if !current.is_empty() || lines.is_empty() {
        lines.push(RenderedLine { spans: current });
    }
    lines
}
