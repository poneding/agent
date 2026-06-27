use std::io::{BufRead, Cursor, Write};

use agent_ui_bridge::UiApprovalRequest;

use crate::{CliOutput, handle_slash_command, render_approval_prompt};

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
pub const fn render_plain_repl_prompt() -> &'static str {
    "agent> "
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
        "/exit" | "/quit" => Some(PlainReplEvent::Exit),
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

pub fn run_plain_repl_io<R, W, F>(mut reader: R, mut writer: W, mut run_prompt: F) -> i32
where
    R: BufRead,
    W: Write,
    F: FnMut(&str) -> Result<String, PlainReplRuntimeError>,
{
    if write_all(&mut writer, "agent plain repl\n").is_err() {
        return 2;
    }

    loop {
        if write_all(&mut writer, render_plain_repl_prompt()).is_err() {
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
            return 0;
        }

        let Some(event) = parse_plain_repl_line(&line) else {
            continue;
        };

        match event {
            PlainReplEvent::PromptSubmitted(prompt) => match run_prompt(&prompt) {
                Ok(output) => {
                    if write_output_line(&mut writer, &output).is_err() {
                        return 2;
                    }
                }
                Err(error) if error.terminal => {
                    let _ = write_all(&mut writer, &format!("terminal error: {}\n", error.message));
                    return 2;
                }
                Err(error) => {
                    if write_all(
                        &mut writer,
                        &format!("recoverable error: {}\n", error.message),
                    )
                    .is_err()
                    {
                        return 2;
                    }
                }
            },
            PlainReplEvent::SlashCommand(command) => {
                let output = handle_slash_command(&command);
                if write_all(&mut writer, &output.stdout).is_err() {
                    return 2;
                }
                if output.exit {
                    return output.status_code;
                }
            }
            PlainReplEvent::Exit => {
                let _ = write_all(&mut writer, "Bye\n");
                return 0;
            }
        }
    }
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
