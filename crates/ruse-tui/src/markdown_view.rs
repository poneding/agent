//! Map ruse-markdown styled lines into ratatui `Line`/`Span` values.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ruse_markdown::{RenderOptions, SegmentRole, render_markdown_lines};

use crate::TerminalTheme;
use crate::palette::{Palette, Semantic};

/// Render Markdown body to ratatui lines using the active terminal theme.
#[must_use]
pub fn markdown_to_lines(
    body: &str,
    base: Style,
    theme: TerminalTheme,
    width: usize,
    streaming: bool,
) -> Vec<Line<'static>> {
    let options = RenderOptions {
        width,
        streaming,
        // OSC 8 sequences are preserved in span text; terminals that support them
        // make links clickable. Others show the styled label text.
        osc8_links: true,
        highlight_code: cfg!(feature = "syntect"),
        code_block_indent: "  ".to_string(),
    };
    let rendered = render_markdown_lines(body, &options);
    if rendered.is_empty() {
        return Vec::new();
    }
    rendered
        .into_iter()
        .map(|line| {
            if line.spans.is_empty() {
                return Line::from("");
            }
            let spans = line
                .spans
                .into_iter()
                .map(|span| Span::styled(span.text, role_style(span.role, base, theme)))
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn role_style(role: SegmentRole, base: Style, theme: TerminalTheme) -> Style {
    let palette = Palette::for_theme(theme);
    match role {
        SegmentRole::Plain => base,
        SegmentRole::Heading(1) => base
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            .fg(palette.heading()),
        SegmentRole::Heading(_) => base.add_modifier(Modifier::BOLD).fg(palette.heading()),
        SegmentRole::Bold => base.add_modifier(Modifier::BOLD),
        SegmentRole::Italic => base.add_modifier(Modifier::ITALIC),
        SegmentRole::Strikethrough => base.add_modifier(Modifier::CROSSED_OUT),
        SegmentRole::Code => base.fg(palette.code()),
        SegmentRole::CodeBlock => base.fg(palette.code_block()),
        SegmentRole::CodeBlockBorder => palette.meta(),
        SegmentRole::Quote => base.add_modifier(Modifier::ITALIC).fg(palette.quote()),
        SegmentRole::QuoteBorder => palette.meta(),
        SegmentRole::ListBullet => base.fg(palette.list_bullet()),
        SegmentRole::Link => base.add_modifier(Modifier::UNDERLINED).fg(palette.link()),
        SegmentRole::LinkUrl => palette.meta(),
        SegmentRole::Hr => palette.meta(),
        SegmentRole::TableBorder => palette.meta(),
        SegmentRole::Highlight { fg, bold, italic } => {
            let mut style = base;
            if let Some((r, g, b)) = fg {
                style = style.fg(Color::Rgb(r, g, b));
            } else {
                style = style.fg(palette.code_block());
            }
            if bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            style
        }
    }
}

/// Reasoning body uses the Reasoning semantic base plus markdown roles.
#[must_use]
pub fn reasoning_markdown_lines(
    body: &str,
    theme: TerminalTheme,
    width: usize,
    streaming: bool,
) -> Vec<Line<'static>> {
    // Only the reasoning body is italic. Headers (Thinking/Thought) stay upright
    // + bold via the Reasoning semantic; body drops bold and adds italic so the
    // aside reads as process notes rather than a status label.
    let base = Palette::for_theme(theme)
        .semantic(Semantic::Reasoning)
        .remove_modifier(Modifier::BOLD)
        .add_modifier(Modifier::ITALIC);
    let lines = markdown_to_lines(body, base, theme, width, streaming);
    // Indent reasoning content as an aside (two spaces), matching the plain path.
    lines
        .into_iter()
        .map(|mut line| {
            let mut spans = vec![Span::styled("  ", base)];
            spans.append(&mut line.spans);
            Line::from(spans)
        })
        .collect()
}
