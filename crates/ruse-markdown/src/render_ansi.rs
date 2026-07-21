//! Render Markdown to an ANSI-styled string for plain CLI output.

use crate::render_lines::render_markdown_lines;
use crate::theme::{RenderOptions, RenderedLine, SegmentRole};

/// Render Markdown source to a multi-line ANSI string.
#[must_use]
pub fn render_markdown_ansi(source: &str, options: &RenderOptions) -> String {
    let lines = render_markdown_lines(source, options);
    lines_to_ansi(&lines)
}

/// Convert styled lines into a single ANSI string (joined by newlines).
#[must_use]
pub fn lines_to_ansi(lines: &[RenderedLine]) -> String {
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for span in &line.spans {
            out.push_str(&style_ansi(&span.text, span.role));
        }
    }
    out
}

fn style_ansi(text: &str, role: SegmentRole) -> String {
    if text.is_empty() {
        return String::new();
    }
    // OSC 8 sequences already embed their own escapes; do not recolor the whole
    // payload when the text starts with an OSC 8 open.
    if text.starts_with("\x1b]8;;") {
        return format!("\x1b[4;34m{text}\x1b[0m");
    }
    match role {
        SegmentRole::Plain => text.to_string(),
        SegmentRole::Heading(1) => wrap(text, "1;4;36"), // bold underline cyan
        SegmentRole::Heading(_) => wrap(text, "1;36"),   // bold cyan
        SegmentRole::Bold => wrap(text, "1"),
        SegmentRole::Italic => wrap(text, "3"),
        SegmentRole::Strikethrough => wrap(text, "9"),
        SegmentRole::Code => wrap(text, "38;5;180"),
        SegmentRole::CodeBlock => wrap(text, "38;5;252"),
        SegmentRole::CodeBlockBorder => wrap(text, "38;5;244"),
        SegmentRole::Quote => wrap(text, "3;38;5;245"),
        SegmentRole::QuoteBorder => wrap(text, "38;5;240"),
        SegmentRole::ListBullet => wrap(text, "38;5;110"),
        SegmentRole::Link => wrap(text, "4;34"),
        SegmentRole::LinkUrl => wrap(text, "38;5;244"),
        SegmentRole::Hr => wrap(text, "38;5;240"),
        SegmentRole::TableBorder => wrap(text, "38;5;240"),
        SegmentRole::Highlight { fg, bold, italic } => {
            let mut codes = Vec::new();
            if bold {
                codes.push("1".to_string());
            }
            if italic {
                codes.push("3".to_string());
            }
            if let Some((r, g, b)) = fg {
                codes.push(format!("38;2;{r};{g};{b}"));
            } else {
                codes.push("38;5;252".to_string());
            }
            wrap(text, &codes.join(";"))
        }
    }
}

fn wrap(text: &str, codes: &str) -> String {
    format!("\x1b[{codes}m{text}\x1b[0m")
}

/// Whether plain-CLI ANSI Markdown rendering is enabled via environment.
///
/// Honors `RUSE_MARKDOWN_ANSI=1|true|yes|on`.
#[must_use]
pub fn ansi_markdown_enabled_from_env() -> bool {
    match std::env::var("RUSE_MARKDOWN_ANSI") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}
