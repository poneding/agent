//! Structured intermediate representation for Markdown documents.

/// A parsed Markdown document as an ordered list of blocks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// Block-level Markdown constructs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Block {
    Paragraph {
        inlines: Vec<Inline>,
    },
    Heading {
        level: u8,
        inlines: Vec<Inline>,
    },
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    BlockQuote {
        blocks: Vec<Block>,
    },
    ThematicBreak,
    Table {
        headers: Vec<TableCell>,
        alignments: Vec<TableAlign>,
        rows: Vec<Vec<TableCell>>,
    },
    /// Raw HTML preserved as plain text for terminal display.
    Html {
        raw: String,
    },
}

/// One list item, which may contain nested blocks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListItem {
    pub blocks: Vec<Block>,
    pub task: Option<bool>,
}

/// Table cell content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableCell {
    pub inlines: Vec<Inline>,
}

/// Column alignment for tables.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TableAlign {
    #[default]
    None,
    Left,
    Center,
    Right,
}

/// Inline-level Markdown constructs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Inline {
    Text(String),
    SoftBreak,
    HardBreak,
    Code(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Link {
        text: Vec<Inline>,
        dest: String,
        title: Option<String>,
    },
    Image {
        alt: String,
        dest: String,
    },
    Html(String),
}
