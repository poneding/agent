//! Streaming-friendly Markdown source normalization.

/// Prepare source text before parsing.
///
/// - Tabs become three spaces (matches pi).
/// - When `streaming` is true, a trailing partial fence line (`\``, `` `` ``, or
///   partial `~~~`) is stripped so incomplete closing fences do not flicker the
///   open code block as the final fence characters stream in.
#[must_use]
pub fn prepare_source(source: &str, streaming: bool) -> String {
    let normalized = source.replace('\t', "   ");
    if !streaming {
        return normalized;
    }
    trim_partial_closing_fence(&normalized)
}

/// Remove a trailing incomplete fence line when it looks like a partial close.
fn trim_partial_closing_fence(source: &str) -> String {
    let Some(last_nl) = source.rfind('\n') else {
        return source.to_string();
    };
    let (head, last_line) = source.split_at(last_nl + 1);
    let trimmed = last_line.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        return source.to_string();
    }
    // A partial fence is only backticks or only tildes, length 1..=2 (full fence
    // is usually 3+). Longer complete fences are left alone.
    let is_backticks = !trimmed.is_empty() && trimmed.chars().all(|c| c == '`');
    let is_tildes = !trimmed.is_empty() && trimmed.chars().all(|c| c == '~');
    if (is_backticks || is_tildes) && trimmed.chars().count() < 3 {
        // Keep the newline after head so the open fence still has a body boundary.
        return head.trim_end_matches('\n').to_string();
    }
    source.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_source_expands_tabs() {
        assert_eq!(prepare_source("a\tb", false), "a   b");
    }

    #[test]
    fn streaming_strips_partial_fence() {
        let src = "```rust\nfn main() {}\n``";
        let prepared = prepare_source(src, true);
        assert!(!prepared.ends_with('`') || prepared.ends_with("```"));
        assert!(prepared.contains("fn main()"));
        assert!(!prepared.ends_with("``"));
    }

    #[test]
    fn streaming_keeps_complete_fence() {
        let src = "```rust\nfn main() {}\n```";
        assert_eq!(prepare_source(src, true), src);
    }
}
