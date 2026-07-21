//! pulldown-cmark event stream → structured [`Document`] IR.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};

use crate::ir::{Block, Document, Inline, ListItem, TableAlign, TableCell};
use crate::stream::prepare_source;

/// Parse Markdown into a structured document IR.
#[must_use]
pub fn parse_markdown(source: &str) -> Document {
    parse_markdown_with_options(source, false)
}

/// Parse Markdown, applying streaming fence normalization when `streaming` is set.
#[must_use]
pub fn parse_markdown_with_options(source: &str, streaming: bool) -> Document {
    let prepared = prepare_source(source, streaming);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&prepared, options);

    let mut root_blocks = Vec::new();
    let mut block_stack: Vec<OpenBlock> = Vec::new();
    let mut inline_stack: Vec<OpenInline> = Vec::new();
    let mut list_stack: Vec<OpenList> = Vec::new();
    let mut table_stack: Vec<OpenTable> = Vec::new();
    let mut current_inlines: Vec<Inline> = Vec::new();
    let mut pending_task: Option<bool> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    // paragraph collects into current_inlines
                }
                Tag::Heading { level, .. } => {
                    block_stack.push(OpenBlock::Heading { level: level as u8 });
                    current_inlines.clear();
                }
                Tag::BlockQuote(_) => {
                    block_stack.push(OpenBlock::Quote { blocks: Vec::new() });
                }
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        CodeBlockKind::Indented => None,
                        CodeBlockKind::Fenced(info) => {
                            let info = info.trim();
                            if info.is_empty() {
                                None
                            } else {
                                Some(info.split_whitespace().next().unwrap_or(info).to_string())
                            }
                        }
                    };
                    block_stack.push(OpenBlock::Code {
                        lang,
                        code: String::new(),
                    });
                }
                Tag::List(start) => {
                    // Nested list under a tight list item: flush bare text first.
                    if let Some(OpenBlock::ListItem { blocks, .. }) = block_stack.last_mut() {
                        flush_loose_inlines(&mut current_inlines, blocks);
                    }
                    list_stack.push(OpenList {
                        ordered: start.is_some(),
                        start: start.unwrap_or(1),
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    block_stack.push(OpenBlock::ListItem {
                        blocks: Vec::new(),
                        task: pending_task.take(),
                    });
                }
                Tag::Emphasis => {
                    inline_stack.push(OpenInline::Emphasis {
                        inlines: std::mem::take(&mut current_inlines),
                    });
                }
                Tag::Strong => {
                    inline_stack.push(OpenInline::Strong {
                        inlines: std::mem::take(&mut current_inlines),
                    });
                }
                Tag::Strikethrough => {
                    inline_stack.push(OpenInline::Strikethrough {
                        inlines: std::mem::take(&mut current_inlines),
                    });
                }
                Tag::Link {
                    dest_url, title, ..
                } => {
                    inline_stack.push(OpenInline::Link {
                        dest: dest_url.into_string(),
                        title: nonempty_title(title),
                        outer: std::mem::take(&mut current_inlines),
                    });
                }
                Tag::Image {
                    dest_url, title, ..
                } => {
                    inline_stack.push(OpenInline::Image {
                        dest: dest_url.into_string(),
                        alt: title.into_string(),
                        outer: std::mem::take(&mut current_inlines),
                    });
                }
                Tag::Table(alignments) => {
                    table_stack.push(OpenTable {
                        alignments: alignments
                            .into_iter()
                            .map(|align| match align {
                                pulldown_cmark::Alignment::None => TableAlign::None,
                                pulldown_cmark::Alignment::Left => TableAlign::Left,
                                pulldown_cmark::Alignment::Center => TableAlign::Center,
                                pulldown_cmark::Alignment::Right => TableAlign::Right,
                            })
                            .collect(),
                        headers: Vec::new(),
                        rows: Vec::new(),
                        current_row: Vec::new(),
                        in_head: false,
                    });
                }
                Tag::TableHead => {
                    if let Some(table) = table_stack.last_mut() {
                        table.in_head = true;
                    }
                }
                Tag::TableRow => {
                    if let Some(table) = table_stack.last_mut() {
                        table.current_row.clear();
                    }
                }
                Tag::TableCell => {
                    current_inlines.clear();
                }
                Tag::HtmlBlock => {
                    block_stack.push(OpenBlock::Html { raw: String::new() });
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => {
                    let block = Block::Paragraph {
                        inlines: std::mem::take(&mut current_inlines),
                    };
                    push_block(&mut root_blocks, &mut block_stack, &mut list_stack, block);
                }
                TagEnd::Heading(_) => {
                    let level = match block_stack.pop() {
                        Some(OpenBlock::Heading { level }) => level,
                        _ => 1,
                    };
                    let block = Block::Heading {
                        level,
                        inlines: std::mem::take(&mut current_inlines),
                    };
                    push_block(&mut root_blocks, &mut block_stack, &mut list_stack, block);
                }
                TagEnd::BlockQuote(_) => {
                    if let Some(OpenBlock::Quote { blocks }) = block_stack.pop() {
                        push_block(
                            &mut root_blocks,
                            &mut block_stack,
                            &mut list_stack,
                            Block::BlockQuote { blocks },
                        );
                    }
                }
                TagEnd::CodeBlock => {
                    if let Some(OpenBlock::Code { lang, code }) = block_stack.pop() {
                        let code = code.trim_end_matches('\n').to_string();
                        push_block(
                            &mut root_blocks,
                            &mut block_stack,
                            &mut list_stack,
                            Block::CodeBlock { lang, code },
                        );
                    }
                }
                TagEnd::List(_) => {
                    if let Some(list) = list_stack.pop() {
                        push_block(
                            &mut root_blocks,
                            &mut block_stack,
                            &mut list_stack,
                            Block::List {
                                ordered: list.ordered,
                                start: list.start,
                                items: list.items,
                            },
                        );
                    }
                }
                TagEnd::Item => {
                    if let Some(OpenBlock::ListItem { mut blocks, task }) = block_stack.pop() {
                        // Tight lists emit bare Text without Paragraph wrappers.
                        flush_loose_inlines(&mut current_inlines, &mut blocks);
                        if let Some(list) = list_stack.last_mut() {
                            list.items.push(ListItem { blocks, task });
                        }
                    }
                }
                TagEnd::Emphasis => {
                    if let Some(OpenInline::Emphasis { inlines: outer }) = inline_stack.pop() {
                        let inner = std::mem::take(&mut current_inlines);
                        current_inlines = outer;
                        current_inlines.push(Inline::Emphasis(inner));
                    }
                }
                TagEnd::Strong => {
                    if let Some(OpenInline::Strong { inlines: outer }) = inline_stack.pop() {
                        let inner = std::mem::take(&mut current_inlines);
                        current_inlines = outer;
                        current_inlines.push(Inline::Strong(inner));
                    }
                }
                TagEnd::Strikethrough => {
                    if let Some(OpenInline::Strikethrough { inlines: outer }) = inline_stack.pop() {
                        let inner = std::mem::take(&mut current_inlines);
                        current_inlines = outer;
                        current_inlines.push(Inline::Strikethrough(inner));
                    }
                }
                TagEnd::Link => {
                    if let Some(OpenInline::Link { dest, title, outer }) = inline_stack.pop() {
                        let text = std::mem::take(&mut current_inlines);
                        current_inlines = outer;
                        current_inlines.push(Inline::Link { text, dest, title });
                    }
                }
                TagEnd::Image => {
                    if let Some(OpenInline::Image { dest, alt, outer }) = inline_stack.pop() {
                        let mut final_alt = alt;
                        let collected = std::mem::take(&mut current_inlines);
                        if final_alt.is_empty() {
                            final_alt = inline_plain_text(&collected);
                        }
                        current_inlines = outer;
                        current_inlines.push(Inline::Image {
                            alt: final_alt,
                            dest,
                        });
                    }
                }
                TagEnd::Table => {
                    if let Some(table) = table_stack.pop() {
                        push_block(
                            &mut root_blocks,
                            &mut block_stack,
                            &mut list_stack,
                            Block::Table {
                                headers: table.headers,
                                alignments: table.alignments,
                                rows: table.rows,
                            },
                        );
                    }
                }
                TagEnd::TableHead => {
                    if let Some(table) = table_stack.last_mut() {
                        // pulldown-cmark emits header cells under TableHead without
                        // a TableRow wrapper — promote current_row to headers here.
                        if !table.current_row.is_empty() {
                            table.headers = std::mem::take(&mut table.current_row);
                        }
                        table.in_head = false;
                    }
                }
                TagEnd::TableRow => {
                    if let Some(table) = table_stack.last_mut() {
                        let row = std::mem::take(&mut table.current_row);
                        if table.in_head {
                            table.headers = row;
                        } else {
                            table.rows.push(row);
                        }
                    }
                }
                TagEnd::TableCell => {
                    if let Some(table) = table_stack.last_mut() {
                        table.current_row.push(TableCell {
                            inlines: std::mem::take(&mut current_inlines),
                        });
                    }
                }
                TagEnd::HtmlBlock => {
                    if let Some(OpenBlock::Html { raw }) = block_stack.pop() {
                        push_block(
                            &mut root_blocks,
                            &mut block_stack,
                            &mut list_stack,
                            Block::Html { raw },
                        );
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if let Some(OpenBlock::Code { code, .. }) = block_stack.last_mut() {
                    code.push_str(&text);
                } else if let Some(OpenBlock::Html { raw }) = block_stack.last_mut() {
                    raw.push_str(&text);
                } else {
                    current_inlines.push(Inline::Text(text.into_string()));
                }
            }
            Event::Code(code) => {
                current_inlines.push(Inline::Code(code.into_string()));
            }
            Event::SoftBreak => current_inlines.push(Inline::SoftBreak),
            Event::HardBreak => current_inlines.push(Inline::HardBreak),
            Event::Html(html) | Event::InlineHtml(html) => {
                if let Some(OpenBlock::Html { raw }) = block_stack.last_mut() {
                    raw.push_str(&html);
                } else {
                    current_inlines.push(Inline::Html(html.into_string()));
                }
            }
            Event::Rule => {
                push_block(
                    &mut root_blocks,
                    &mut block_stack,
                    &mut list_stack,
                    Block::ThematicBreak,
                );
            }
            Event::TaskListMarker(checked) => {
                pending_task = Some(checked);
                if let Some(OpenBlock::ListItem { task, .. }) = block_stack.last_mut() {
                    *task = Some(checked);
                }
            }
            Event::FootnoteReference(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {}
        }
    }

    Document {
        blocks: root_blocks,
    }
}

enum OpenBlock {
    Heading {
        level: u8,
    },
    Quote {
        blocks: Vec<Block>,
    },
    Code {
        lang: Option<String>,
        code: String,
    },
    ListItem {
        blocks: Vec<Block>,
        task: Option<bool>,
    },
    Html {
        raw: String,
    },
}

enum OpenInline {
    Emphasis {
        inlines: Vec<Inline>,
    },
    Strong {
        inlines: Vec<Inline>,
    },
    Strikethrough {
        inlines: Vec<Inline>,
    },
    Link {
        dest: String,
        title: Option<String>,
        outer: Vec<Inline>,
    },
    Image {
        dest: String,
        alt: String,
        outer: Vec<Inline>,
    },
}

struct OpenList {
    ordered: bool,
    start: u64,
    items: Vec<ListItem>,
}

struct OpenTable {
    alignments: Vec<TableAlign>,
    headers: Vec<TableCell>,
    rows: Vec<Vec<TableCell>>,
    current_row: Vec<TableCell>,
    in_head: bool,
}

fn nonempty_title(title: CowStr<'_>) -> Option<String> {
    let title = title.into_string();
    if title.is_empty() { None } else { Some(title) }
}

fn push_block(
    root: &mut Vec<Block>,
    block_stack: &mut [OpenBlock],
    _list_stack: &mut [OpenList],
    block: Block,
) {
    match block_stack.last_mut() {
        Some(OpenBlock::Quote { blocks }) | Some(OpenBlock::ListItem { blocks, .. }) => {
            blocks.push(block);
        }
        _ => root.push(block),
    }
}

fn flush_loose_inlines(current_inlines: &mut Vec<Inline>, blocks: &mut Vec<Block>) {
    if current_inlines.is_empty() {
        return;
    }
    blocks.push(Block::Paragraph {
        inlines: std::mem::take(current_inlines),
    });
}

fn inline_plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(text) | Inline::Code(text) | Inline::Html(text) => out.push_str(text),
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            Inline::Strong(inner) | Inline::Emphasis(inner) | Inline::Strikethrough(inner) => {
                out.push_str(&inline_plain_text(inner))
            }
            Inline::Link { text, .. } => out.push_str(&inline_plain_text(text)),
            Inline::Image { alt, .. } => out.push_str(alt),
        }
    }
    out
}
