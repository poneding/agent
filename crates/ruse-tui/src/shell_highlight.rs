//! Lightweight shell-command syntax highlighting for tool rows.
//!
//! One-line commands next to `Running...` / `Ran` are tokenized into command
//! names, flags, strings, operators, variables, and plain args — then colored
//! from the active palette (not a full shell grammar; good enough for display).

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

use crate::TerminalTheme;
use crate::palette::Palette;

/// Semantic token kinds for a displayed shell command line.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShellTokenKind {
    /// First word of a pipeline segment (`ls`, `cargo`, `grep`).
    Command,
    /// Option/flag (`-a`, `--all`, `--flag=value`).
    Flag,
    /// Quoted string (`'…'`, `"…"`).
    String,
    /// Shell operator / redirect (`|`, `&&`, `||`, `;`, `>`, `>>`, `<`, `&`).
    Operator,
    /// Parameter expansion (`$HOME`, `${PATH}`).
    Variable,
    /// `#` comment to end of line.
    Comment,
    /// Everything else: args, paths, whitespace, assignments.
    Text,
}

/// One token from a shell command display string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellToken<'a> {
    pub text: &'a str,
    pub kind: ShellTokenKind,
}

/// Tokenize a single-line shell command for display highlighting.
#[must_use]
pub fn tokenize_shell_command(command: &str) -> Vec<ShellToken<'_>> {
    let bytes = command.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    // Next non-operator word is a command name (start of line / after | && || ;).
    let mut expect_command = true;

    while i < bytes.len() {
        // Whitespace — keep as Text so layout is preserved.
        if bytes[i].is_ascii_whitespace() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(ShellToken {
                text: &command[start..i],
                kind: ShellTokenKind::Text,
            });
            continue;
        }

        // Comment to EOL.
        if bytes[i] == b'#' {
            tokens.push(ShellToken {
                text: &command[i..],
                kind: ShellTokenKind::Comment,
            });
            break;
        }

        // Quoted string.
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                // Allow \" inside double quotes; \' is not special in single quotes
                // in real shells, but escaping \" is common enough to skip.
                if quote == b'"' && bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            tokens.push(ShellToken {
                text: &command[start..i],
                kind: ShellTokenKind::String,
            });
            expect_command = false;
            continue;
        }

        // Variable: $NAME or ${...}
        if bytes[i] == b'$' {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'{' {
                i += 1;
                while i < bytes.len() && bytes[i] != b'}' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1; // closing }
                }
            } else {
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
            }
            tokens.push(ShellToken {
                text: &command[start..i],
                kind: ShellTokenKind::Variable,
            });
            expect_command = false;
            continue;
        }

        // Multi-char then single-char operators / redirects.
        if let Some(len) = operator_len(&bytes[i..]) {
            tokens.push(ShellToken {
                text: &command[i..i + len],
                kind: ShellTokenKind::Operator,
            });
            // After pipe / list separators, the next word is a new command.
            let op = &command[i..i + len];
            expect_command = matches!(op, "|" | "||" | "&&" | ";" | "&");
            i += len;
            continue;
        }

        // Bare word: flag, command, or arg (may include =, ., /, : etc.).
        let start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'\''
            && bytes[i] != b'"'
            && bytes[i] != b'#'
            && bytes[i] != b'$'
            && operator_len(&bytes[i..]).is_none()
        {
            i += 1;
        }
        let word = &command[start..i];
        let kind = classify_word(word, expect_command);
        if matches!(kind, ShellTokenKind::Command) {
            expect_command = false;
        } else if expect_command && is_env_assignment(word) {
            // FOO=bar prefix keeps looking for the real command.
        } else if expect_command {
            expect_command = false;
        }
        tokens.push(ShellToken { text: word, kind });
    }

    tokens
}

/// Highlight a shell command into ratatui spans using the active theme palette.
///
/// All tokens are italic (aside cue next to the status verb); colors carry the
/// syntax roles. Falls back to a single muted italic span when empty.
#[must_use]
pub fn highlight_shell_command(command: &str, theme: TerminalTheme) -> Vec<Span<'static>> {
    if command.is_empty() {
        return Vec::new();
    }
    let palette = Palette::for_theme(theme);
    tokenize_shell_command(command)
        .into_iter()
        .map(|token| {
            Span::styled(
                token.text.to_string(),
                shell_token_style(token.kind, palette, theme),
            )
        })
        .collect()
}

fn shell_token_style(kind: ShellTokenKind, palette: Palette, theme: TerminalTheme) -> Style {
    // Italic on every token so the whole command reads as a typed line, not a path.
    let italic = Style::default().add_modifier(Modifier::ITALIC);
    match kind {
        // Command name: keyword orange + bold — the "verb" of the line.
        ShellTokenKind::Command => italic.fg(palette.code()).add_modifier(Modifier::BOLD),
        // Flags: cyan-blue tag.
        ShellTokenKind::Flag => italic.fg(palette.list_bullet()),
        // Strings: syntax string green.
        ShellTokenKind::String => italic.fg(string_color(theme)),
        // Operators / redirects: muted.
        ShellTokenKind::Operator => italic.fg(palette.quote()),
        // Variables: entity/heading blue.
        ShellTokenKind::Variable => italic.fg(palette.heading()),
        // Comments: muted meta (already italic via Semantic::Meta).
        ShellTokenKind::Comment => palette.meta(),
        // Paths / args / whitespace: muted secondary.
        ShellTokenKind::Text => italic.fg(palette.quote()),
    }
}

fn string_color(theme: TerminalTheme) -> ratatui::style::Color {
    use ratatui::style::Color;
    match theme {
        TerminalTheme::HighContrast | TerminalTheme::System => Color::Green,
        TerminalTheme::Light => Color::Rgb(0x86, 0xB3, 0x00),
        TerminalTheme::Dark => Color::Rgb(0xAA, 0xD9, 0x4C),
    }
}

fn classify_word(word: &str, expect_command: bool) -> ShellTokenKind {
    if word.is_empty() {
        return ShellTokenKind::Text;
    }
    // Flags: -a, --all, --flag=value (but not `-` alone as stdin, keep as text).
    if word.starts_with('-') && word != "-" && word != "--" {
        return ShellTokenKind::Flag;
    }
    if expect_command && !is_env_assignment(word) {
        return ShellTokenKind::Command;
    }
    ShellTokenKind::Text
}

fn is_env_assignment(word: &str) -> bool {
    // FOO=bar at command position (leading assignment).
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn operator_len(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }
    // Longest match first.
    let two = if bytes.len() >= 2 {
        Some(&bytes[..2])
    } else {
        None
    };
    match two {
        Some(b"||" | b"&&" | b">>" | b"<<" | b"2>" | b">&") => return Some(2),
        _ => {}
    }
    match bytes[0] {
        b'|' | b'&' | b';' | b'>' | b'<' | b'(' | b')' => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(cmd: &str) -> Vec<ShellTokenKind> {
        tokenize_shell_command(cmd)
            .into_iter()
            .filter(|t| !t.text.chars().all(|c| c.is_whitespace()))
            .map(|t| t.kind)
            .collect()
    }

    fn texts(cmd: &str) -> Vec<&str> {
        tokenize_shell_command(cmd)
            .into_iter()
            .filter(|t| !t.text.chars().all(|c| c.is_whitespace()))
            .map(|t| t.text)
            .collect()
    }

    #[test]
    fn ls_flag_splits_command_and_flag() {
        assert_eq!(
            kinds("ls -a"),
            vec![ShellTokenKind::Command, ShellTokenKind::Flag]
        );
        assert_eq!(texts("ls -a"), vec!["ls", "-a"]);
    }

    #[test]
    fn long_flags_and_args() {
        assert_eq!(
            kinds("ls -la /tmp"),
            vec![
                ShellTokenKind::Command,
                ShellTokenKind::Flag,
                ShellTokenKind::Text
            ]
        );
        assert_eq!(
            kinds("rg -n pattern src/"),
            vec![
                ShellTokenKind::Command,
                ShellTokenKind::Flag,
                ShellTokenKind::Text,
                ShellTokenKind::Text,
            ]
        );
    }

    #[test]
    fn double_dash_alone_is_text_not_flag() {
        // `--` ends option parsing; keep as text so it doesn't steal flag color.
        let tokens = tokenize_shell_command("cargo test -- --nocapture");
        let non_ws: Vec<_> = tokens
            .into_iter()
            .filter(|t| !t.text.chars().all(|c| c.is_whitespace()))
            .collect();
        assert_eq!(non_ws[0].text, "cargo");
        assert_eq!(non_ws[0].kind, ShellTokenKind::Command);
        assert_eq!(non_ws[1].text, "test");
        assert_eq!(non_ws[1].kind, ShellTokenKind::Text);
        assert_eq!(non_ws[2].text, "--");
        assert_eq!(non_ws[2].kind, ShellTokenKind::Text);
        assert_eq!(non_ws[3].text, "--nocapture");
        assert_eq!(non_ws[3].kind, ShellTokenKind::Flag);
    }

    #[test]
    fn pipeline_resets_command() {
        assert_eq!(
            kinds("ls -la | grep foo"),
            vec![
                ShellTokenKind::Command,
                ShellTokenKind::Flag,
                ShellTokenKind::Operator,
                ShellTokenKind::Command,
                ShellTokenKind::Text,
            ]
        );
    }

    #[test]
    fn strings_and_variables() {
        assert_eq!(
            kinds(r#"echo "hello" $HOME"#),
            vec![
                ShellTokenKind::Command,
                ShellTokenKind::String,
                ShellTokenKind::Variable,
            ]
        );
        assert_eq!(
            kinds("echo 'x'"),
            vec![ShellTokenKind::Command, ShellTokenKind::String]
        );
    }

    #[test]
    fn env_assignment_then_command() {
        assert_eq!(
            kinds("FOO=bar ls -a"),
            vec![
                ShellTokenKind::Text,
                ShellTokenKind::Command,
                ShellTokenKind::Flag,
            ]
        );
    }

    #[test]
    fn comment_consumes_rest() {
        assert_eq!(
            kinds("ls # list files"),
            vec![ShellTokenKind::Command, ShellTokenKind::Comment]
        );
    }

    #[test]
    fn highlight_produces_spans_matching_text() {
        let spans = highlight_shell_command("ls -a", TerminalTheme::Dark);
        let joined: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(joined, "ls -a");
        assert!(spans.len() >= 3); // ls, space, -a
        // Command span is bold+italic; flag is italic.
        assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(spans[0].style.add_modifier.contains(Modifier::ITALIC));
    }
}
