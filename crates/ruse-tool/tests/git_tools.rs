use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ruse_sandbox::WorkspaceSandbox;
use ruse_tool::builtins::{GitDiffTool, GitStatusTool};
use ruse_tool::{Tool, ToolInput, ToolName, ToolRegistry};
use serde_json::json;
use tempfile::TempDir;

fn git_repo_fixture() -> (TempDir, WorkspaceSandbox) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    run_git(&root, ["init", "-q"]);
    run_git(&root, ["config", "user.name", "Agent"]);
    run_git(&root, ["config", "user.email", "agent@example.invalid"]);
    fs::write(root.join("README.md"), "old text\n").expect("readme");
    run_git(&root, ["add", "README.md"]);
    run_git(&root, ["commit", "-m", "initial", "--quiet"]);
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    (temp, sandbox)
}

fn non_git_fixture() -> (TempDir, WorkspaceSandbox) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    (temp, sandbox)
}

fn workspace_root(temp: &TempDir) -> PathBuf {
    temp.path().join("workspace")
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn git_tools_status_returns_empty_files_for_clean_repo() {
    let (_temp, sandbox) = git_repo_fixture();
    let tool = GitStatusTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({ "cwd": "." }),
        })
        .expect("git status");

    assert_eq!(output.value, json!({ "files": [] }));
}

#[test]
fn git_tools_status_reports_modified_files() {
    let (temp, sandbox) = git_repo_fixture();
    fs::write(workspace_root(&temp).join("README.md"), "new text\n").expect("modify readme");
    let tool = GitStatusTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({ "cwd": "." }),
        })
        .expect("git status");

    assert_eq!(
        output.value,
        json!({
            "files": [
                { "path": "README.md", "status": "modified" }
            ]
        })
    );
}

#[test]
fn git_tools_diff_returns_text_for_modified_file() {
    let (temp, sandbox) = git_repo_fixture();
    fs::write(workspace_root(&temp).join("README.md"), "new text\n").expect("modify readme");
    let tool = GitDiffTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "cwd": ".",
                "path": "README.md"
            }),
        })
        .expect("git diff");
    let diff = output.value["diff"].as_str().expect("diff text");

    assert!(diff.contains("--- a/README.md"));
    assert!(diff.contains("+++ b/README.md"));
    assert!(diff.contains("-old text"));
    assert!(diff.contains("+new text"));
}

#[test]
fn git_tools_status_returns_execution_error_for_non_git_directory() {
    let (_temp, sandbox) = non_git_fixture();
    let tool = GitStatusTool::new(sandbox);

    let error = tool
        .execute(ToolInput {
            value: json!({ "cwd": "." }),
        })
        .expect_err("non-git directory rejected");

    assert!(error.to_string().contains("git status failed"));
}

#[test]
fn git_tools_status_rejects_cwd_traversal() {
    let (_temp, sandbox) = git_repo_fixture();
    let tool = GitStatusTool::new(sandbox);

    let error = tool
        .execute(ToolInput {
            value: json!({ "cwd": ".." }),
        })
        .expect_err("cwd traversal rejected");

    assert!(error.to_string().contains("path escapes workspace"));
}

#[test]
fn git_tools_register_in_registry() {
    let (_temp, sandbox) = git_repo_fixture();
    let mut registry = ToolRegistry::default();

    registry
        .register(GitStatusTool::new(sandbox.clone()))
        .expect("register status");
    registry
        .register(GitDiffTool::new(sandbox))
        .expect("register diff");

    assert!(
        registry
            .get(&ToolName::new("workspace.git_status").expect("status name"))
            .is_some()
    );
    assert!(
        registry
            .get(&ToolName::new("workspace.git_diff").expect("diff name"))
            .is_some()
    );
}
