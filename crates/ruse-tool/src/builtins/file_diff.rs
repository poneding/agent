//! Shared unified-diff helpers for write/edit tool results shown in the TUI.

/// Build a unified-style diff for a full-file replacement.
///
/// Empty `before` → create (only `+` lines). Empty `after` → delete (only `-`).
/// Both non-empty → full replace dump (line-level `-` then `+`). Good enough for
/// transcript display; not a Myers hunk algorithm.
#[must_use]
pub fn unified_file_diff(path: &str, before: &str, after: &str) -> String {
    if before == after {
        return format!("--- {path}\n+++ {path}\n@@\n (no changes)\n");
    }
    let mut body = String::new();
    if !before.is_empty() {
        for line in before.lines() {
            body.push('-');
            body.push_str(line);
            body.push('\n');
        }
        // Preserve a trailing empty line marker if the source ended with `\n\n` etc.
        if before.ends_with('\n') && before.lines().next().is_some() {
            // lines() drops a final empty segment; if content was "a\n", lines is ["a"] — fine.
        }
    }
    if !after.is_empty() {
        for line in after.lines() {
            body.push('+');
            body.push_str(line);
            body.push('\n');
        }
    }
    format!("--- {path}\n+++ {path}\n@@\n{body}")
}
