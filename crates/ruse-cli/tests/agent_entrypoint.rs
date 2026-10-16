use std::fs;
use std::path::PathBuf;
use std::process::Command;

use ruse_cli::{
    CliCommand, CliInvocation, InteractiveMode, ModelSelection, OutputMode, StdioProfile,
    parse_cli_command, parse_cli_invocation, render_cli_invocation_with_stdio,
};
use ruse_core::ConfigOverride;
use ruse_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid json frame line"))
        .collect()
}

fn fake_config_path() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agent-entrypoint-fake-config-{}.yaml",
        std::process::id()
    ));
    fs::write(
        &path,
        r#"
schemaVersion: 1
model:
  defaultProvider: fake
  invocation:
    timeoutMs: 30000
    maxAttempts: 1
    retryableFailures: []
    fallbackProviderIds: []
  providers:
    - id: fake
      kind: fake
      defaultModel: fake-scripted
      enabled: true
      timeoutMs: 1000
"#,
    )
    .expect("write fake config");
    path
}

#[test]
fn agent_entrypoint_parses_tui_and_plain_repl_commands() {
    assert_eq!(
        parse_cli_command(["tui"]).expect("parse tui"),
        CliCommand::TuiShell
    );
    assert_eq!(
        parse_cli_command(["repl", "--plain"]).expect("parse plain repl"),
        CliCommand::PlainRepl
    );
}

#[test]
fn agent_entrypoint_selects_tui_for_interactive_no_args() {
    let invocation = parse_cli_invocation(std::iter::empty::<&str>()).expect("parse no args");
    let output = render_cli_invocation_with_stdio(
        invocation,
        StdioProfile {
            stdin_tty: true,
            stdout_tty: true,
        },
    );

    assert!(!output.exit);
    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("agent tui shell ready"));
    assert!(!output.stdout.contains("GPT 5.5 xhigh"));
    assert!(!output.stdout.contains("❯ "));
}

#[test]
fn agent_entrypoint_fails_non_tty_no_args_with_help() {
    let invocation = parse_cli_invocation(std::iter::empty::<&str>()).expect("parse no args");
    let output = render_cli_invocation_with_stdio(
        invocation,
        StdioProfile {
            stdin_tty: false,
            stdout_tty: false,
        },
    );

    assert!(output.exit);
    assert_ne!(output.status_code, 0);
    assert!(output.stdout.contains("usage: agent"));
    assert!(output.stdout.contains("agent --prompt"));
}

#[test]
fn agent_entrypoint_preserves_json_prompt_script_mode() {
    let invocation =
        parse_cli_invocation(["--json", "--prompt", "hello"]).expect("parse json prompt");

    assert_eq!(
        invocation,
        CliInvocation {
            command: CliCommand::BatchPrompt {
                prompt: "hello".to_string(),
            },
            output_mode: OutputMode::Json,
            model_selection: ModelSelection::default(),
            config_path: None,
            config_override: ConfigOverride::default(),
            permission_mode: Default::default(),
        }
    );

    let mut invocation = invocation;
    invocation.config_path = Some(fake_config_path());
    let output = render_cli_invocation_with_stdio(
        invocation,
        StdioProfile {
            stdin_tty: true,
            stdout_tty: true,
        },
    );
    let frames = parse_frames(&output.stdout);

    assert!(output.exit);
    assert_eq!(output.status_code, 0);
    assert!(!output.stdout.contains("agent tui"));
    assert!(matches!(frames.first(), Some(UiJsonFrame::Event(_))));
    assert!(matches!(frames.last(), Some(UiJsonFrame::Done { .. })));
}

#[test]
fn agent_binary_runs_version_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse"))
        .arg("--version")
        .output()
        .expect("agent binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("agent "));
}

#[cfg(unix)]
#[test]
fn agent_binary_no_args_enters_and_restores_alternate_screen_under_pty() {
    if Command::new("python3").arg("--version").output().is_err() {
        return;
    }

    let home = tempfile::tempdir().expect("temp home");
    let script = r#"
import os
import pty
import selectors
import struct
import subprocess
import sys
import time
import fcntl
import termios

agent = sys.argv[1]
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
env = os.environ.copy()
env["TERM"] = "xterm-256color"

process = subprocess.Popen(
    [agent],
    stdin=slave,
    stdout=slave,
    stderr=slave,
    env=env,
    close_fds=True,
)
os.close(slave)

selector = selectors.DefaultSelector()
selector.register(master, selectors.EVENT_READ)
output = bytearray()
started = time.monotonic()
deadline = started + 6.0
sent_exit = False
alternate_screen_seen_at = None

while time.monotonic() < deadline:
    if alternate_screen_seen_at is None and b"\x1b[?1049h" in output:
        alternate_screen_seen_at = time.monotonic()
    if (
        not sent_exit
        and (
            alternate_screen_seen_at is not None
            and time.monotonic() - alternate_screen_seen_at >= 0.25
            or time.monotonic() - started >= 2.0
        )
    ):
        os.write(master, b"/exit\r")
        sent_exit = True

    for key, _ in selector.select(0.05):
        try:
            chunk = os.read(master, 8192)
        except OSError:
            chunk = b""
        if not chunk:
            break
        output.extend(chunk)

    if process.poll() is not None:
        break

if process.poll() is None:
    try:
        os.write(master, b"\x03")
    except OSError:
        pass
    try:
        process.wait(timeout=1)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=1)

try:
    while True:
        chunk = os.read(master, 8192)
        if not chunk:
            break
        output.extend(chunk)
except OSError:
    pass
os.close(master)

sys.stdout.buffer.write(output)
sys.exit(process.returncode if process.returncode is not None else 124)
"#;

    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_ruse"))
        .env("HOME", home.path())
        .env_remove("AGENT_CONFIG")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GEMINI_API_KEY")
        .output()
        .expect("pty smoke should run");

    assert!(
        output.status.success(),
        "pty smoke exited with {status:?}; stdout={stdout:?}; stderr={stderr:?}",
        status = output.status.code(),
        stdout = String::from_utf8_lossy(&output.stdout),
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[?1049h"),
        "alternate screen enter sequence missing from pty output: {stdout:?}"
    );
    assert!(
        stdout.contains("\u{1b}[?1049l"),
        "alternate screen leave sequence missing from pty output: {stdout:?}"
    );
    assert!(
        stdout.contains("fake-scripted") || stdout.contains("agent"),
        "terminal app did not render recognizable status text: {stdout:?}"
    );
}

#[test]
fn legacy_agent_cli_binary_still_runs_version_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .arg("--version")
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn interactive_mode_contract_is_public_and_stable() {
    assert_eq!(InteractiveMode::Auto, InteractiveMode::Auto);
    assert_eq!(InteractiveMode::Tui, InteractiveMode::Tui);
    assert_eq!(InteractiveMode::Plain, InteractiveMode::Plain);
}
