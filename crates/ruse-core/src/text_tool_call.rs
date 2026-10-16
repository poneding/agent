//! Recover tool calls that models emit as plain text instead of native
//! structured tool-use events.
//!
//! History reconstruction currently serializes tools as:
//! `tool_call id=… name=… input={…}`
//! Weak models (or providers that drop tool schemas) will *imitate* that string
//! as assistant prose. Without recovery, the TUI shows the raw dump and never
//! runs the tool.

use serde_json::Value;

/// One tool call recovered from assistant text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextToolCall {
    pub call_id: String,
    pub tool_name: String,
    pub input_json: Value,
}

/// Split assistant text into (remaining_prose, recovered_tool_calls).
///
/// Recognizes lines (or runs) matching:
/// - `tool_call id=… name=… input={…}`
/// - loose variants with extra whitespace / repeated dumps
#[must_use]
pub fn extract_text_tool_calls(text: &str) -> (String, Vec<TextToolCall>) {
    if text.trim().is_empty() {
        return (String::new(), Vec::new());
    }

    let mut calls = Vec::new();
    let mut prose_parts = Vec::new();
    let mut seen_ids = std::collections::BTreeSet::new();

    // Normalize accidental concatenation / repeated dumps.
    let normalized = text
        .replace("}tool_call", "}\ntool_call")
        .replace("} tool_call", "}\ntool_call")
        .replace("tool_call\nid=", "tool_call id=");

    for raw_line in normalized.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            // Keep blank lines only if we already have prose (paragraph spacing).
            if !prose_parts.is_empty() {
                prose_parts.push(String::new());
            }
            continue;
        }

        // Line may contain multiple `tool_call` dumps jammed together.
        if line.contains("tool_call") && line.contains("name=") && line.contains("input=") {
            let recovered = parse_tool_call_line_or_run(line);
            if !recovered.is_empty() {
                for call in recovered {
                    if seen_ids.insert(call.call_id.clone()) {
                        calls.push(call);
                    }
                }
                continue;
            }
        }

        // Drop pure wire-format echoes that failed to parse fully.
        if looks_like_tool_call_noise(line) {
            continue;
        }

        prose_parts.push(raw_line.to_string());
    }

    // Trim trailing blank prose lines.
    while prose_parts
        .last()
        .is_some_and(|line| line.trim().is_empty())
    {
        prose_parts.pop();
    }
    // Trim leading blank prose lines.
    while prose_parts
        .first()
        .is_some_and(|line| line.trim().is_empty())
    {
        prose_parts.remove(0);
    }

    let prose = prose_parts.join("\n");
    (prose, calls)
}

/// True when the whole message is just tool-call wire format (no useful prose).
#[must_use]
pub fn is_tool_call_only_text(text: &str) -> bool {
    let (prose, calls) = extract_text_tool_calls(text);
    !calls.is_empty() && prose.trim().is_empty()
}

fn looks_like_tool_call_noise(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    (lower.starts_with("tool_call") || lower.contains("tool_call id="))
        && (lower.contains("name=") || lower.contains("input="))
}

fn parse_tool_call_line_or_run(line: &str) -> Vec<TextToolCall> {
    let mut out = Vec::new();
    // Split on repeated "tool_call" starts.
    let mut rest = line;
    while let Some(idx) = find_tool_call_start(rest) {
        let chunk_start = &rest[idx..];
        let next = find_tool_call_start(&chunk_start[1..])
            .map(|i| i + 1)
            .unwrap_or(chunk_start.len());
        let chunk = chunk_start[..next].trim();
        if let Some(call) = parse_one_tool_call(chunk) {
            out.push(call);
        }
        rest = &chunk_start[next..];
        if rest.is_empty() {
            break;
        }
    }
    out
}

fn find_tool_call_start(s: &str) -> Option<usize> {
    s.find("tool_call")
}

fn parse_one_tool_call(chunk: &str) -> Option<TextToolCall> {
    // tool_call id=CALL name=NAME input={JSON}
    let rest = chunk.trim().strip_prefix("tool_call")?.trim_start();
    let id = extract_field(rest, "id=")?;
    let name = extract_field(rest, "name=")?;
    let input_raw = extract_json_after(rest, "input=")?;
    let input_json: Value = serde_json::from_str(input_raw.trim()).ok()?;
    if !input_json.is_object() {
        return None;
    }
    let call_id = id.trim().to_string();
    let tool_name = name.trim().to_string();
    if call_id.is_empty() || tool_name.is_empty() {
        return None;
    }
    Some(TextToolCall {
        call_id,
        tool_name,
        input_json,
    })
}

fn extract_field<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let start = s.find(key)? + key.len();
    let tail = &s[start..];
    // Field ends at next known key or end.
    let end = [" id=", " name=", " input=", " tool_call"]
        .iter()
        .filter_map(|marker| {
            // Avoid matching the current key's own position at 0.
            tail.find(marker).map(|i| i)
        })
        .filter(|i| *i > 0)
        .min()
        .unwrap_or(tail.len());
    let value = tail[..end].trim();
    (!value.is_empty()).then_some(value)
}

fn extract_json_after<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let start = s.find(key)? + key.len();
    let tail = s[start..].trim_start();
    if !tail.starts_with('{') {
        return None;
    }
    // Brace-match the JSON object.
    let bytes = tail.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&tail[..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_single_tool_call_and_clears_prose() {
        let text = r#"tool_call id=call_1 name=workspace.shell_command input={"program":"ls -a"}"#;
        let (prose, calls) = extract_text_tool_calls(text);
        assert!(prose.trim().is_empty(), "{prose}");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_id, "call_1");
        assert_eq!(calls[0].tool_name, "workspace.shell_command");
        assert_eq!(calls[0].input_json, json!({"program": "ls -a"}));
    }

    #[test]
    fn extracts_jammed_duplicates_once() {
        let text = r#"tool_call id=call_17 name=workspace.shell_command input={"program":"ls -a"} tool_call
id=call_17 name=workspace.shell_command input={"program":"ls -a"}"#;
        let (prose, calls) = extract_text_tool_calls(text);
        assert!(
            prose.trim().is_empty() || !prose.contains("tool_call"),
            "{prose}"
        );
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert_eq!(calls[0].call_id, "call_17");
    }

    #[test]
    fn keeps_real_prose_around_tool_calls() {
        let text = "I'll list the files.\ntool_call id=c1 name=workspace.shell_command input={\"program\":\"ls\"}\nDone.";
        let (prose, calls) = extract_text_tool_calls(text);
        assert!(prose.contains("I'll list the files."), "{prose}");
        assert!(prose.contains("Done."), "{prose}");
        assert!(!prose.contains("tool_call"), "{prose}");
        assert_eq!(calls.len(), 1);
    }
}
