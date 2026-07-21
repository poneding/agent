//! Golden fixtures: Markdown source → IR shape + plain rendered text.

use ruse_markdown::{
    Block, Inline, RenderOptions, parse_markdown, parse_markdown_with_options,
    render_markdown_lines,
};

fn plain_render(source: &str) -> String {
    let lines = render_markdown_lines(
        source,
        &RenderOptions {
            width: 80,
            streaming: false,
            osc8_links: false,
            highlight_code: false,
            code_block_indent: "  ".to_string(),
        },
    );
    lines
        .into_iter()
        .map(|line| line.plain_text())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn golden_basic_ir_and_render() {
    let source = include_str!("fixtures/basic.md");
    let doc = parse_markdown(source);
    assert!(matches!(
        doc.blocks.first(),
        Some(Block::Heading { level: 1, .. })
    ));
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b, Block::Paragraph { .. }))
    );

    let rendered = plain_render(source);
    assert!(rendered.contains("Hello"));
    assert!(rendered.contains("bold"));
    assert!(rendered.contains("italic"));
    assert!(rendered.contains("strike"));
    assert!(rendered.contains("code"));
    assert!(rendered.contains("Subheading"));
    assert!(rendered.contains("link"));
    assert!(rendered.contains("https://example.com"));
}

#[test]
fn golden_lists() {
    let source = include_str!("fixtures/lists.md");
    let doc = parse_markdown(source);
    let list_count = doc
        .blocks
        .iter()
        .filter(|b| matches!(b, Block::List { .. }))
        .count();
    assert!(list_count >= 1);

    let rendered = plain_render(source);
    assert!(rendered.contains("- alpha") || rendered.contains("alpha"));
    assert!(rendered.contains("one") && rendered.contains("two"));
    assert!(rendered.contains("[x]") || rendered.contains("done"));
}

#[test]
fn golden_code_block() {
    let source = include_str!("fixtures/code.md");
    let doc = parse_markdown(source);
    let code = doc.blocks.iter().find_map(|b| match b {
        Block::CodeBlock { lang, code } => Some((lang.as_deref(), code.as_str())),
        _ => None,
    });
    let (lang, body) = code.expect("fenced code block");
    assert_eq!(lang, Some("rust"));
    assert!(body.contains("fn main()"));

    let rendered = plain_render(source);
    assert!(rendered.contains("```rust"));
    assert!(rendered.contains("println!"));
    assert!(rendered.contains("```"));
}

#[test]
fn golden_table() {
    let source = include_str!("fixtures/table.md");
    let doc = parse_markdown(source);
    assert!(
        doc.blocks.iter().any(|b| matches!(b, Block::Table { .. })),
        "expected table block, got {:?}",
        doc.blocks
    );
    let rendered = plain_render(source);
    assert!(rendered.contains("Ada"));
    assert!(rendered.contains("Name"));
}

#[test]
fn golden_quote_and_hr() {
    let source = include_str!("fixtures/quote_hr.md");
    let doc = parse_markdown(source);
    assert!(
        doc.blocks
            .iter()
            .any(|b| matches!(b, Block::BlockQuote { .. }))
    );
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::ThematicBreak)));
    let rendered = plain_render(source);
    assert!(rendered.contains("│") || rendered.contains("quoted"));
    assert!(rendered.contains("─"));
    assert!(rendered.contains("After the break"));
}

#[test]
fn inline_strong_emphasis_code() {
    let doc = parse_markdown("**a** *b* `c`");
    let Block::Paragraph { inlines } = &doc.blocks[0] else {
        panic!("expected paragraph");
    };
    assert!(inlines.iter().any(|i| matches!(i, Inline::Strong(_))));
    assert!(inlines.iter().any(|i| matches!(i, Inline::Emphasis(_))));
    assert!(inlines.iter().any(|i| matches!(i, Inline::Code(_))));
}

#[test]
fn streaming_partial_fence_keeps_code_open() {
    let partial = "```js\nconsole.log(1)\n``";
    let complete = "```js\nconsole.log(1)\n```";
    let streaming = parse_markdown_with_options(partial, true);
    let finished = parse_markdown_with_options(complete, false);
    // Streaming partial should still produce a code block (not swallow as text).
    assert!(
        streaming
            .blocks
            .iter()
            .any(|b| matches!(b, Block::CodeBlock { .. })),
        "streaming partial fence should parse as code block: {:?}",
        streaming.blocks
    );
    assert!(
        finished
            .blocks
            .iter()
            .any(|b| matches!(b, Block::CodeBlock { lang: Some(l), .. } if l == "js"))
    );
}

#[test]
fn width_wrap_respects_limit() {
    let source = "word ".repeat(40);
    let lines = render_markdown_lines(
        source.trim(),
        &RenderOptions {
            width: 40,
            streaming: false,
            osc8_links: false,
            highlight_code: false,
            code_block_indent: "  ".to_string(),
        },
    );
    for line in &lines {
        let text = line.plain_text();
        if text.trim().is_empty() {
            continue;
        }
        assert!(
            unicode_width::UnicodeWidthStr::width(text.as_str()) <= 40,
            "line too wide: {text:?}"
        );
    }
}

#[test]
fn ansi_render_includes_escape_codes_for_emphasis() {
    let ansi = ruse_markdown::render_markdown_ansi(
        "**bold** and `code`",
        &RenderOptions {
            width: 80,
            streaming: false,
            osc8_links: false,
            highlight_code: false,
            code_block_indent: "  ".to_string(),
        },
    );
    assert!(ansi.contains('\u{1b}'), "expected ANSI escapes: {ansi:?}");
    assert!(ansi.contains("bold"));
    assert!(ansi.contains("code"));
}

#[test]
fn osc8_link_embed() {
    let lines = render_markdown_lines(
        "[x](https://example.com)",
        &RenderOptions {
            width: 80,
            streaming: false,
            osc8_links: true,
            highlight_code: false,
            code_block_indent: "  ".to_string(),
        },
    );
    let text = lines.iter().map(|l| l.plain_text()).collect::<String>();
    assert!(text.contains("\u{1b}]8;;https://example.com"), "{text:?}");
}
