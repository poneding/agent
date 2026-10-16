use std::fs;
use std::path::PathBuf;
use std::process::Command;

use ruse_sandbox::{ProcessSandboxMode, WorkspaceSandbox};
use ruse_tool::builtins::ShellCommandTool;
use ruse_tool::{Tool, ToolInput};
use serde_json::json;
use tempfile::TempDir;

fn sandbox_fixture() -> (TempDir, WorkspaceSandbox, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::create_dir(root.join("subdir")).expect("subdir");
    // Compile helper into the workspace so absolute program paths stay confined.
    let helper = compile_helper(&root);
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    (temp, sandbox, helper)
}

fn compile_helper(root: &std::path::Path) -> PathBuf {
    let source = root.join("shell_helper.rs");
    fs::write(
        &source,
        r#"
use std::env;
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    match env::args().nth(1).as_deref() {
        Some("stdout") => print!("hello stdout\n"),
        Some("stderr") => eprint!("hello stderr\n"),
        Some("exit7") => {
            print!("before exit\n");
            process::exit(7);
        }
        Some("sleep") => thread::sleep(Duration::from_secs(5)),
        Some("cwd") => print!("{}", env::current_dir().expect("cwd").display()),
        Some("env") => print!("{}", env::var("AGENT_ALLOWED").unwrap_or_default()),
        _ => process::exit(64),
    }
}
"#,
    )
    .expect("write helper source");
    let binary = root.join(format!("shell-helper{}", std::env::consts::EXE_SUFFIX));
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let status = Command::new(rustc)
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .status()
        .expect("compile helper");
    assert!(status.success(), "helper program should compile");
    binary
}

#[test]
fn shell_command_tool_captures_successful_stdout_in_confined_cwd() {
    let (temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);
    let expected_cwd = temp
        .path()
        .join("workspace/subdir")
        .canonicalize()
        .expect("canonical cwd")
        .display()
        .to_string();

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["cwd"],
                "cwd": "subdir",
                "timeout_ms": 30000,
                "env": {}
            }),
        })
        .expect("execute command");

    assert_eq!(
        output.value,
        json!({
            "status": 0,
            "stdout": expected_cwd,
            "stderr": "",
            "timed_out": false
        })
    );
}

#[test]
fn shell_command_tool_captures_stderr() {
    let (_temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["stderr"],
                "cwd": ".",
                "timeout_ms": 30000,
                "env": {}
            }),
        })
        .expect("execute command");

    assert_eq!(
        output.value,
        json!({
            "status": 0,
            "stdout": "",
            "stderr": "hello stderr\n",
            "timed_out": false
        })
    );
}

#[test]
fn shell_command_tool_returns_non_zero_status_as_data() {
    let (_temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["exit7"],
                "cwd": ".",
                "timeout_ms": 30000,
                "env": {}
            }),
        })
        .expect("non-zero status is data");

    assert_eq!(
        output.value,
        json!({
            "status": 7,
            "stdout": "before exit\n",
            "stderr": "",
            "timed_out": false
        })
    );
}

#[test]
fn shell_command_tool_times_out_and_kills_process() {
    let (_temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["sleep"],
                "cwd": ".",
                "timeout_ms": 10,
                "env": {}
            }),
        })
        .expect("timeout is returned as data");

    assert_eq!(output.value["timed_out"], true);
    assert_eq!(output.value["status"], -1);
}

#[test]
fn shell_command_tool_rejects_cwd_traversal() {
    let (_temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let error = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["stdout"],
                "cwd": "..",
                "timeout_ms": 30000,
                "env": {}
            }),
        })
        .expect_err("cwd traversal rejected");

    assert!(error.to_string().contains("path escapes workspace"));
}

#[test]
fn shell_command_tool_rejects_env_keys_that_are_not_allowlisted() {
    let (_temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let error = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["env"],
                "cwd": ".",
                "timeout_ms": 30000,
                "env": { "AGENT_ALLOWED": "visible" }
            }),
        })
        .expect_err("env key rejected");

    assert!(
        error
            .to_string()
            .contains("environment key is not allowlisted")
    );
}

#[test]
fn shell_command_tool_passes_allowlisted_environment_values() {
    let (_temp, sandbox, helper) = sandbox_fixture();
    let tool = ShellCommandTool::with_allowed_env(sandbox, ["AGENT_ALLOWED"])
        .with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": helper,
                "args": ["env"],
                "cwd": ".",
                "timeout_ms": 30000,
                "env": { "AGENT_ALLOWED": "visible" }
            }),
        })
        .expect("execute command");

    assert_eq!(
        output.value,
        json!({
            "status": 0,
            "stdout": "visible",
            "stderr": "",
            "timed_out": false
        })
    );
}

#[test]
fn shell_command_tool_rejects_program_path_outside_workspace() {
    let (_temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let err = tool
        .execute(ToolInput {
            value: json!({
                "program": "/bin/echo",
                "args": ["hi"],
                "cwd": ".",
                "timeout_ms": 30000,
                "env": {}
            }),
        })
        .expect_err("absolute program outside workspace must be rejected");

    let message = err.to_string();
    assert!(
        message.contains("outside the workspace") || message.contains("escapes"),
        "unexpected error: {message}"
    );
}

#[test]
fn shell_command_tool_hides_sandbox_exec_from_missing_program_stderr() {
    let (_temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PreferSeatbelt);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": "ruse_definitely_missing_binary_xyz",
                "args": [],
                "cwd": ".",
                "timeout_ms": 10000,
                "env": {}
            }),
        })
        .expect("missing program should still return a tool result");

    let stderr = output
        .value
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        !stderr.contains("sandbox-exec"),
        "stderr must not leak sandbox-exec: {stderr}"
    );
    assert!(
        !stderr.contains("execvp"),
        "stderr must not leak execvp diagnostics: {stderr}"
    );
    assert!(
        stderr.contains("ruse_definitely_missing_binary_xyz"),
        "stderr should still name the missing program: {stderr}"
    );
    let status = output
        .value
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    assert_ne!(status, 0, "missing program must be a non-zero exit");
}

#[test]
fn shell_command_tool_seatbelt_denies_write_outside_workspace_on_macos() {
    let (temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PreferSeatbelt);
    let outside = temp.path().join("outside.txt");
    let outside_str = outside.display().to_string();

    // /bin/bash is absolute outside workspace path form - use bare sh via PATH
    let output = tool.execute(ToolInput {
        value: json!({
            "program": "sh",
            "args": ["-c", format!("echo leaked > '{outside_str}'")],
            "cwd": ".",
            "timeout_ms": 10000,
            "env": {}
        }),
    });

    #[cfg(target_os = "macos")]
    {
        // Seatbelt should prevent the write; command may fail or leave file absent.
        let _ = output; // may be Ok with non-zero or Err
        assert!(
            !outside.exists(),
            "seatbelt must block write outside workspace"
        );
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (output, outside);
    }
}

#[test]
fn shell_command_accepts_space_separated_program_as_argv() {
    // Model mistake: program="ls -la" with empty args (was "No such file").
    let (_temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": "ls",
                "args": ["-la", "."],
                "cwd": ".",
                "timeout_ms": 10000,
                "env": {}
            }),
        })
        .expect("ls -la should run");

    let status = output
        .value
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    assert_eq!(status, 0, "status={status} value={}", output.value);
    let stdout = output
        .value
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        !stdout.is_empty(),
        "ls should list workspace: stdout={stdout:?}"
    );
}

#[test]
fn shell_command_normalizes_combined_program_string() {
    let (_temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": "ls -la",
                "args": [],
                "cwd": ".",
                "timeout_ms": 10000,
                "env": {}
            }),
        })
        .expect("combined program string should normalize to ls + args");

    let status = output
        .value
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    assert_eq!(status, 0, "status={status} value={}", output.value);
}

#[test]
fn shell_command_runs_shell_snippet_with_and() {
    let (_temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": "pwd && ls",
                "args": [],
                "cwd": ".",
                "timeout_ms": 10000,
                "env": {}
            }),
        })
        .expect("shell snippet should run via sh -c");

    let status = output
        .value
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    assert_eq!(status, 0, "status={status} value={}", output.value);
    let stdout = output
        .value
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(!stdout.trim().is_empty(), "stdout empty: {stdout:?}");
}

#[test]
fn shell_command_splits_program_even_when_args_also_present() {
    // Model sends program="ls -a" AND args=["."] — must not exec "ls -a" as a name.
    let (_temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox).with_process_mode(ProcessSandboxMode::PathOnly);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "program": "ls -a",
                "args": ["."],
                "cwd": ".",
                "timeout_ms": 10000,
                "env": {}
            }),
        })
        .expect("program with spaces must split before exec");

    let status = output
        .value
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    let stderr = output
        .value
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        status, 0,
        "status={status} stderr={stderr} value={}",
        output.value
    );
}
