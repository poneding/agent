//! Semantic style roles for Markdown rendering.

/// A semantic role applied to a rendered text span.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SegmentRole {
    Plain,
    Heading(u8),
    Bold,
    Italic,
    Strikethrough,
    Code,
    CodeBlock,
    CodeBlockBorder,
    Quote,
    QuoteBorder,
    ListBullet,
    Link,
    LinkUrl,
    Hr,
    TableBorder,
    /// Syntax-highlighted token with optional truecolor RGB.
    Highlight {
        fg: Option<(u8, u8, u8)>,
        bold: bool,
        italic: bool,
    },
}

/// One styled piece of text on a rendered line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyledSpan {
    pub text: String,
    pub role: SegmentRole,
}

/// One rendered terminal line of styled spans.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderedLine {
    pub spans: Vec<StyledSpan>,
}

impl RenderedLine {
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            spans: vec![StyledSpan {
                text: text.into(),
                role: SegmentRole::Plain,
            }],
        }
    }

    #[must_use]
    pub fn single(text: impl Into<String>, role: SegmentRole) -> Self {
        Self {
            spans: vec![StyledSpan {
                text: text.into(),
                role,
            }],
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() || self.spans.iter().all(|s| s.text.is_empty())
    }

    #[must_use]
    pub fn plain_text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
}

/// Options controlling Markdown → lines / ANSI rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    /// Maximum visible columns for wrapping. `0` means unlimited (no wrap).
    pub width: usize,
    /// Apply streaming fence normalization before parse.
    pub streaming: bool,
    /// Emit OSC 8 hyperlinks for Markdown links when the terminal supports them.
    pub osc8_links: bool,
    /// Attempt syntect highlighting when the `syntect` feature is enabled.
    pub highlight_code: bool,
    /// Indent for fenced code block body lines.
    pub code_block_indent: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 0,
            streaming: false,
            osc8_links: true,
            highlight_code: cfg!(feature = "syntect"),
            code_block_indent: "  ".to_string(),
        }
    }
}

impl RenderOptions {
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }
}
