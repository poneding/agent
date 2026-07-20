use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_sandbox::{ProcessSandboxMode, WorkspaceSandbox};
use agent_tool::builtins::ShellCommandTool;
use agent_tool::{Tool, ToolInput};
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
    let binary = root
        .join(format!("shell-helper{}", std::env::consts::EXE_SUFFIX));
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
            "timed_out": false,
            "sandbox_profile": "workspace_path"
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
            "timed_out": false,
            "sandbox_profile": "workspace_path"
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
            "timed_out": false,
            "sandbox_profile": "workspace_path"
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
            "timed_out": false,
            "sandbox_profile": "workspace_path"
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
        message.contains("outside the workspace sandbox") || message.contains("escapes"),
        "unexpected error: {message}"
    );
}

#[test]
fn shell_command_tool_seatbelt_denies_write_outside_workspace_on_macos() {
    let (temp, sandbox, _helper) = sandbox_fixture();
    let tool = ShellCommandTool::new(sandbox)
        .with_process_mode(ProcessSandboxMode::PreferSeatbelt);
    let outside = temp.path().join("outside.txt");
    let outside_str = outside.display().to_string();

    // /bin/bash is absolute outside workspace path form - use bare sh via PATH
    let output = tool
        .execute(ToolInput {
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
