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
            .fg(heading_color(theme)),
        SegmentRole::Heading(_) => base.add_modifier(Modifier::BOLD).fg(heading_color(theme)),
        SegmentRole::Bold => base.add_modifier(Modifier::BOLD),
        SegmentRole::Italic => base.add_modifier(Modifier::ITALIC),
        SegmentRole::Strikethrough => base.add_modifier(Modifier::CROSSED_OUT),
        SegmentRole::Code => base.fg(code_color(theme)),
        SegmentRole::CodeBlock => base.fg(code_block_color(theme)),
        SegmentRole::CodeBlockBorder => palette.meta(),
        SegmentRole::Quote => base.add_modifier(Modifier::ITALIC).fg(quote_color(theme)),
        SegmentRole::QuoteBorder => palette.meta(),
        SegmentRole::ListBullet => base.fg(list_bullet_color(theme)),
        SegmentRole::Link => base
            .add_modifier(Modifier::UNDERLINED)
            .fg(link_color(theme)),
        SegmentRole::LinkUrl => palette.meta(),
        SegmentRole::Hr => palette.meta(),
        SegmentRole::TableBorder => palette.meta(),
        SegmentRole::Highlight { fg, bold, italic } => {
            let mut style = base;
            if let Some((r, g, b)) = fg {
                style = style.fg(Color::Rgb(r, g, b));
            } else {
                style = style.fg(code_block_color(theme));
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

fn heading_color(theme: TerminalTheme) -> Color {
    match theme {
        TerminalTheme::Light => Color::Rgb(0, 105, 145),
        TerminalTheme::Dark => Color::Rgb(56, 189, 248),
        TerminalTheme::HighContrast => Color::Yellow,
        TerminalTheme::System => Color::Cyan,
    }
}

fn code_color(theme: TerminalTheme) -> Color {
    match theme {
        TerminalTheme::Light => Color::Rgb(180, 83, 9),
        TerminalTheme::Dark => Color::Rgb(251, 191, 36),
        TerminalTheme::HighContrast => Color::Yellow,
        TerminalTheme::System => Color::Yellow,
    }
}

fn code_block_color(theme: TerminalTheme) -> Color {
    match theme {
        TerminalTheme::Light => Color::Rgb(51, 65, 85),
        TerminalTheme::Dark => Color::Rgb(226, 232, 240),
        TerminalTheme::HighContrast => Color::White,
        TerminalTheme::System => Color::Gray,
    }
}

fn quote_color(theme: TerminalTheme) -> Color {
    match theme {
        TerminalTheme::Light => Color::Rgb(100, 116, 139),
        TerminalTheme::Dark => Color::Rgb(148, 163, 184),
        TerminalTheme::HighContrast => Color::Gray,
        TerminalTheme::System => Color::Gray,
    }
}

fn list_bullet_color(theme: TerminalTheme) -> Color {
    match theme {
        TerminalTheme::Light => Color::Rgb(0, 105, 145),
        TerminalTheme::Dark => Color::Rgb(125, 211, 252),
        TerminalTheme::HighContrast => Color::Yellow,
        TerminalTheme::System => Color::Cyan,
    }
}

fn link_color(theme: TerminalTheme) -> Color {
    match theme {
        TerminalTheme::Light => Color::Rgb(37, 99, 235),
        TerminalTheme::Dark => Color::Rgb(96, 165, 250),
        TerminalTheme::HighContrast => Color::Yellow,
        TerminalTheme::System => Color::Blue,
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
    let base = Palette::for_theme(theme).semantic(Semantic::Reasoning);
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
