//! Markdown parsing and terminal-oriented rendering for ruse.
//!
//! Presentation-only: parse Markdown into a structured IR, then render to
//! styled lines (for ratatui) or ANSI text (for plain CLI). Runtime events keep
//! raw Markdown source.

#![forbid(unsafe_code)]

mod highlight;
mod ir;
mod parse;
mod render_ansi;
mod render_lines;
mod stream;
mod theme;

pub use ir::{Block, Document, Inline, ListItem, TableAlign, TableCell};
pub use parse::{parse_markdown, parse_markdown_with_options};
pub use render_ansi::{ansi_markdown_enabled_from_env, lines_to_ansi, render_markdown_ansi};
pub use render_lines::{render_document_lines, render_markdown_lines};
pub use stream::prepare_source;
pub use theme::{RenderOptions, RenderedLine, SegmentRole, StyledSpan};

/// Whether TUI/desktop Markdown rendering should be disabled (plain text).
///
/// Honors `RUSE_MARKDOWN=0|false|off|plain|disabled`.
#[must_use]
pub fn markdown_enabled_from_env() -> bool {
    match std::env::var("RUSE_MARKDOWN") {
        Ok(value) => !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "plain" | "disabled" | "no"
        ),
        Err(_) => true,
    }
}
