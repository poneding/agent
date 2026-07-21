//! Optional syntect-based code highlighting.

#[cfg(feature = "syntect")]
use crate::theme::StyledSpan;
use crate::theme::{RenderedLine, SegmentRole};

/// Highlight a fenced code block into styled lines.
///
/// Without the `syntect` feature this always returns monochrome `CodeBlock` lines.
#[must_use]
pub fn highlight_code_lines(code: &str, lang: Option<&str>, enabled: bool) -> Vec<RenderedLine> {
    if !enabled {
        return plain_code_lines(code);
    }
    #[cfg(feature = "syntect")]
    {
        match highlight_with_syntect(code, lang) {
            Some(lines) => lines,
            None => plain_code_lines(code),
        }
    }
    #[cfg(not(feature = "syntect"))]
    {
        let _ = lang;
        plain_code_lines(code)
    }
}

fn plain_code_lines(code: &str) -> Vec<RenderedLine> {
    if code.is_empty() {
        return vec![RenderedLine::single(String::new(), SegmentRole::CodeBlock)];
    }
    code.split('\n')
        .map(|line| RenderedLine::single(line.to_string(), SegmentRole::CodeBlock))
        .collect()
}

#[cfg(feature = "syntect")]
fn highlight_with_syntect(code: &str, lang: Option<&str>) -> Option<Vec<RenderedLine>> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Style, ThemeSet};
    use syntect::parsing::SyntaxSet;
    use syntect::util::as_24_bit_terminal_escaped;

    // Keep a process-wide syntax/theme set.
    use std::sync::OnceLock;
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

    let syntax_set = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let theme_set = THEME_SET.get_or_init(ThemeSet::load_defaults);
    let theme = theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.values().next())?;

    let syntax = lang
        .and_then(|name| {
            syntax_set
                .find_syntax_by_token(name)
                .or_else(|| syntax_set.find_syntax_by_extension(name))
        })
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    // If language was requested but we only got plain text, still color as code block mono.
    if lang.is_some() && syntax.name == "Plain Text" {
        return None;
    }

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();
    for line in code.split('\n') {
        let ranges = highlighter.highlight_line(line, syntax_set).ok()?;
        let mut spans = Vec::new();
        for (style, text) in ranges {
            spans.push(style_to_span(style, text));
        }
        if spans.is_empty() {
            spans.push(StyledSpan {
                text: String::new(),
                role: SegmentRole::CodeBlock,
            });
        }
        // Silence unused import if we never call the escape helper.
        let _ = as_24_bit_terminal_escaped as fn(&[(Style, &str)], bool) -> String;
        lines.push(RenderedLine { spans });
    }
    Some(lines)
}

#[cfg(feature = "syntect")]
fn style_to_span(style: syntect::highlighting::Style, text: &str) -> StyledSpan {
    let fg = style.foreground;
    StyledSpan {
        text: text.to_string(),
        role: SegmentRole::Highlight {
            fg: Some((fg.r, fg.g, fg.b)),
            bold: style
                .font_style
                .contains(syntect::highlighting::FontStyle::BOLD),
            italic: style
                .font_style
                .contains(syntect::highlighting::FontStyle::ITALIC),
        },
    }
}
