use std::io::{BufRead, Cursor, Write};

use ruse_ui_bridge::UiApprovalRequest;

use crate::{CliOutput, handle_slash_command, render_approval_prompt};

const PLAIN_REPL_SEPARATOR_CHAR: char = '─';
const PLAIN_REPL_DEFAULT_STATUS: &str = "GPT 5.5 xhigh";
const PLAIN_REPL_DEFAULT_WIDTH: usize = 80;
const ANSI_ALT_SCREEN_ENTER: &str = "\x1b[?1049h\x1b[2J\x1b[H";
const ANSI_ALT_SCREEN_LEAVE: &str = "\x1b[?1049l";
const ANSI_CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
const ANSI_RESET: &str = "\x1b[0m";

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum PlainReplSymbolMode {
    #[default]
    Unicode,
    NoEmoji,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlainReplEvent {
    PromptSubmitted(String),
    SlashCommand(String),
    Exit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlainReplRuntimeError {
    pub message: String,
    pub terminal: bool,
}

/// Plain-REPL transcript row. Kinds are a **subset** of
/// [`ruse_ui_bridge::UiTranscriptKind`] (user/assistant/system/error only) —
/// plain REPL is model-only and never emits tool/approval/reasoning rows.
#[derive(Clone, Debug, Eq, PartialEq)]
enum PlainReplTranscriptEntry {
    User(String),
    Assistant(String),
    System(String),
    Error(String),
}

impl PlainReplTranscriptEntry {
    #[must_use]
    fn kind(&self) -> ruse_ui_bridge::UiTranscriptKind {
        match self {
            Self::User(_) => ruse_ui_bridge::UiTranscriptKind::User,
            Self::Assistant(_) => ruse_ui_bridge::UiTranscriptKind::Assistant,
            Self::System(_) => ruse_ui_bridge::UiTranscriptKind::System,
            Self::Error(_) => ruse_ui_bridge::UiTranscriptKind::Error,
        }
    }
}

impl PlainReplRuntimeError {
    #[must_use]
    pub fn recoverable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            terminal: false,
        }
    }

    #[must_use]
    pub fn terminal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            terminal: true,
        }
    }
}

#[must_use]
pub fn render_plain_repl_prompt() -> String {
    render_plain_repl_prompt_with_width(PLAIN_REPL_DEFAULT_WIDTH)
}

#[must_use]
pub fn render_plain_repl_prompt_with_width(width: usize) -> String {
    format!("{}\n❯ ", render_plain_repl_separator(width))
}

#[must_use]
pub fn render_plain_repl_footer(status: &str) -> String {
    render_plain_repl_footer_with_width(status, PLAIN_REPL_DEFAULT_WIDTH)
}

#[must_use]
pub fn render_plain_repl_footer_with_width(status: &str, width: usize) -> String {
    render_plain_repl_footer_with_width_and_symbols(status, width, plain_repl_symbol_mode())
}

#[must_use]
pub fn render_plain_repl_footer_with_width_and_symbols(
    status: &str,
    width: usize,
    symbols: PlainReplSymbolMode,
) -> String {
    format!(
        "{}\n{}\n",
        render_plain_repl_separator(width),
        status_line(status, symbols)
    )
}

#[must_use]
pub fn render_plain_repl_frame(status: &str, width: usize) -> String {
    render_plain_repl_frame_with_symbols(status, width, plain_repl_symbol_mode())
}

#[must_use]
pub fn render_plain_repl_frame_with_symbols(
    status: &str,
    width: usize,
    symbols: PlainReplSymbolMode,
) -> String {
    format!(
        "{}\n❯ \n{}",
        render_plain_repl_separator(width),
        render_plain_repl_footer_with_width_and_symbols(status, width, symbols)
    )
}

#[must_use]
pub fn render_plain_repl_interactive_frame(status: &str, width: usize) -> String {
    render_plain_repl_interactive_frame_with_symbols(status, width, plain_repl_symbol_mode())
}

#[must_use]
pub fn render_plain_repl_interactive_frame_with_symbols(
    status: &str,
    width: usize,
    symbols: PlainReplSymbolMode,
) -> String {
    format!(
        "{}\n❯ \x1b[s\n{}\n{}\n\x1b[u",
        render_plain_repl_separator(width),
        render_plain_repl_separator(width),
        status_line(status, symbols)
    )
}

#[must_use]
pub fn render_plain_repl_interactive_screen(status: &str, width: usize) -> String {
    render_plain_repl_interactive_screen_with_entries(&[], status, width)
}

#[must_use]
pub fn render_plain_repl_separator(width: usize) -> String {
    PLAIN_REPL_SEPARATOR_CHAR.to_string().repeat(width.max(1))
}

#[must_use]
pub fn render_plain_repl_approval_prompt(request: &UiApprovalRequest) -> String {
    render_approval_prompt(request)
}

#[must_use]
pub fn parse_plain_repl_line(line: &str) -> Option<PlainReplEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    match line {
        "/exit" | "/quit" | "/bye" => Some(PlainReplEvent::Exit),
        slash if slash.starts_with('/') => Some(PlainReplEvent::SlashCommand(slash.to_string())),
        prompt => Some(PlainReplEvent::PromptSubmitted(prompt.to_string())),
    }
}

#[must_use]
pub fn render_plain_repl_script<F>(input: &str, run_prompt: F) -> CliOutput
where
    F: FnMut(&str) -> Result<String, PlainReplRuntimeError>,
{
    let reader = Cursor::new(input.as_bytes());
    let mut stdout = Vec::new();
    let status_code = run_plain_repl_io(reader, &mut stdout, run_prompt);
    let stdout = String::from_utf8(stdout).expect("plain repl output is utf-8");

    if status_code == 0 {
        CliOutput::exit(stdout)
    } else {
        CliOutput::failure(stdout, status_code)
    }
}

pub fn run_plain_repl_io<R, W, F>(mut reader: R, mut writer: W, run_prompt: F) -> i32
where
    R: BufRead,
    W: Write,
    F: FnMut(&str) -> Result<String, PlainReplRuntimeError>,
{
    run_plain_repl_io_with_status(
        &mut reader,
        &mut writer,
        PLAIN_REPL_DEFAULT_STATUS,
        run_prompt,
    )
}

pub fn run_plain_repl_io_with_status<R, W, F>(
    mut reader: R,
    mut writer: W,
    status: &str,
    run_prompt: F,
) -> i32
where
    R: BufRead,
    W: Write,
    F: FnMut(&str) -> Result<String, PlainReplRuntimeError>,
{
    run_plain_repl_io_with_status_mode(&mut reader, &mut writer, status, false, run_prompt)
}

pub fn run_plain_repl_io_with_status_mode<R, W, F>(
    mut reader: R,
    mut writer: W,
    status: &str,
    interactive: bool,
    mut run_prompt: F,
) -> i32
where
    R: BufRead,
    W: Write,
    F: FnMut(&str) -> Result<String, PlainReplRuntimeError>,
{
    let mut transcript = Vec::new();
    if interactive && write_all(&mut writer, ANSI_ALT_SCREEN_ENTER).is_err() {
        return 2;
    }

    loop {
        let width = plain_repl_terminal_width(interactive);
        let prompt = if interactive {
            render_plain_repl_interactive_screen_with_entries(&transcript, status, width)
        } else {
            render_plain_repl_prompt_with_width(width)
        };
        if write_all(&mut writer, &prompt).is_err() {
            return 2;
        }
        if writer.flush().is_err() {
            return 2;
        }

        let mut line = String::new();
        let bytes_read = match reader.read_line(&mut line) {
            Ok(bytes_read) => bytes_read,
            Err(error) => {
                let _ = write_all(&mut writer, &format!("terminal error: {error}\n"));
                return 2;
            }
        };
        if bytes_read == 0 {
            if interactive && write_all(&mut writer, ANSI_ALT_SCREEN_LEAVE).is_err() {
                return 2;
            }
            return 0;
        }

        if !interactive && write_all(&mut writer, "\n").is_err() {
            return 2;
        }

        let Some(event) = parse_plain_repl_line(&line) else {
            continue;
        };

        if !interactive
            && write_all(
                &mut writer,
                &render_plain_repl_footer_with_width(status, width),
            )
            .is_err()
        {
            return 2;
        }

        match event {
            PlainReplEvent::PromptSubmitted(prompt) => {
                if interactive {
                    transcript.push(PlainReplTranscriptEntry::User(prompt.clone()));
                    if write_all(
                        &mut writer,
                        &render_plain_repl_interactive_screen_with_entries(
                            &transcript,
                            status,
                            width,
                        ),
                    )
                    .is_err()
                    {
                        return 2;
                    }
                    if writer.flush().is_err() {
                        return 2;
                    }
                }

                match run_prompt(&prompt) {
                    Ok(output) => {
                        if interactive {
                            transcript.push(PlainReplTranscriptEntry::Assistant(output));
                        } else if write_output_line(&mut writer, &output).is_err() {
                            return 2;
                        }
                    }
                    Err(error) if error.terminal => {
                        if interactive {
                            transcript.push(PlainReplTranscriptEntry::Error(format!(
                                "terminal error: {}",
                                error.message
                            )));
                            let _ = write_all(
                                &mut writer,
                                &render_plain_repl_interactive_screen_with_entries(
                                    &transcript,
                                    status,
                                    width,
                                ),
                            );
                            let _ = write_all(&mut writer, ANSI_ALT_SCREEN_LEAVE);
                        } else {
                            let _ = write_all(
                                &mut writer,
                                &format!("terminal error: {}\n", error.message),
                            );
                        }
                        return 2;
                    }
                    Err(error) => {
                        if interactive {
                            transcript.push(PlainReplTranscriptEntry::Error(format!(
                                "recoverable error: {}",
                                error.message
                            )));
                        } else if write_all(
                            &mut writer,
                            &format!("recoverable error: {}\n", error.message),
                        )
                        .is_err()
                        {
                            return 2;
                        }
                    }
                }
            }
            PlainReplEvent::SlashCommand(command) => {
                let output = handle_slash_command(&command);
                if interactive {
                    transcript.push(PlainReplTranscriptEntry::System(format!(
                        "{command}\n{}",
                        output.stdout.trim_end()
                    )));
                } else if write_all(&mut writer, &output.stdout).is_err() {
                    return 2;
                }
                if output.exit {
                    if interactive {
                        let _ = write_all(&mut writer, ANSI_ALT_SCREEN_LEAVE);
                    }
                    return output.status_code;
                }
            }
            PlainReplEvent::Exit => {
                if interactive {
                    let _ = write_all(&mut writer, ANSI_ALT_SCREEN_LEAVE);
                } else {
                    let _ = write_all(&mut writer, "Bye\n");
                }
                return 0;
            }
        }
    }
}

fn plain_repl_terminal_width(interactive: bool) -> usize {
    if interactive
        && let Ok((width, _height)) = crossterm::terminal::size()
        && width > 0
    {
        return usize::from(width);
    }

    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width > 0)
        .unwrap_or(PLAIN_REPL_DEFAULT_WIDTH)
}

fn status_line(status: &str, symbols: PlainReplSymbolMode) -> String {
    match symbols {
        PlainReplSymbolMode::Unicode => format!(" 🧠 {status}"),
        PlainReplSymbolMode::NoEmoji => format!(" [model] {status}"),
    }
}

fn plain_repl_symbol_mode() -> PlainReplSymbolMode {
    let no_emoji = std::env::var("AGENT_NO_EMOJI")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        || std::env::var("TERM")
            .ok()
            .is_some_and(|value| value == "dumb");
    if no_emoji {
        PlainReplSymbolMode::NoEmoji
    } else {
        PlainReplSymbolMode::Unicode
    }
}

fn render_plain_repl_interactive_screen_with_entries(
    entries: &[PlainReplTranscriptEntry],
    status: &str,
    width: usize,
) -> String {
    let mut output = String::from(ANSI_CLEAR_SCREEN);
    for entry in entries {
        output.push_str(&render_plain_repl_transcript_entry(entry, width));
    }
    output.push_str(&render_plain_repl_interactive_frame(status, width));
    output
}

fn render_plain_repl_transcript_entry(entry: &PlainReplTranscriptEntry, width: usize) -> String {
    debug_assert!(
        entry.kind().plain_repl_subset(),
        "plain REPL must not emit tool/approval rows"
    );
    match entry {
        PlainReplTranscriptEntry::User(message) => render_user_transcript_entry(message, width),
        PlainReplTranscriptEntry::Assistant(message) => {
            let body = if ruse_markdown::ansi_markdown_enabled_from_env() {
                ruse_markdown::render_markdown_ansi(
                    message.trim_end(),
                    &ruse_markdown::RenderOptions {
                        width,
                        streaming: false,
                        osc8_links: true,
                        highlight_code: true,
                        code_block_indent: "  ".to_string(),
                    },
                )
            } else {
                render_wrapped_lines(message.trim_end(), width)
            };
            format!("Agent\n{body}\n\n")
        }
        PlainReplTranscriptEntry::System(message) => format!(
            "\x1b[38;5;244m{}\n\n{ANSI_RESET}",
            render_wrapped_lines(message.trim_end(), width)
        ),
        PlainReplTranscriptEntry::Error(message) => format!(
            "\x1b[38;5;203m{}\n\n{ANSI_RESET}",
            render_wrapped_lines(message.trim_end(), width)
        ),
    }
}

fn render_user_transcript_entry(message: &str, width: usize) -> String {
    let mut output = format!("\x1b[48;5;236m\x1b[38;5;231m You {ANSI_RESET}\n");
    let wrapped = render_wrapped_lines(message, width.saturating_sub(2));
    for line in wrapped.lines() {
        output.push_str(&format!(
            "\x1b[48;5;236m\x1b[38;5;231m {line} {ANSI_RESET}\n"
        ));
    }
    output.push('\n');
    output
}

fn render_wrapped_lines(text: &str, width: usize) -> String {
    let limit = width.max(20);
    let mut output = String::new();
    for line in text.lines() {
        if line.is_empty() {
            output.push('\n');
            continue;
        }
        let mut current_width = 0;
        for ch in line.chars() {
            let ch_width = if ch.is_ascii() { 1 } else { 2 };
            if current_width + ch_width > limit {
                output.push('\n');
                current_width = 0;
            }
            output.push(ch);
            current_width += ch_width;
        }
        output.push('\n');
    }
    output.trim_end_matches('\n').to_string()
}

fn write_output_line<W>(writer: &mut W, output: &str) -> std::io::Result<()>
where
    W: Write,
{
    write_all(writer, output)?;
    if !output.ends_with('\n') {
        write_all(writer, "\n")?;
    }
    Ok(())
}

fn write_all<W>(writer: &mut W, output: &str) -> std::io::Result<()>
where
    W: Write,
{
    writer.write_all(output.as_bytes())
}
