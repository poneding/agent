use std::fs;

use agent_sandbox::WorkspaceSandbox;
use agent_tool::builtins::{FileEditTool, WriteFileTool};
use agent_tool::{Tool, ToolInput, ToolName, ToolRegistry};
use serde_json::json;
use tempfile::tempdir;

fn sandbox_fixture() -> (tempfile::TempDir, WorkspaceSandbox) {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(root.join("README.md"), "old text\n").expect("readme");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    (temp, sandbox)
}

#[test]
fn file_edit_preview_returns_diff_without_mutating_file() {
    let (temp, sandbox) = sandbox_fixture();
    let tool = FileEditTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "path": "README.md",
                "expected": "old text\n",
                "replacement": "new text\n",
                "apply": false
            }),
        })
        .expect("preview edit");

    assert_eq!(
        output.value,
        json!({
            "path": "README.md",
            "applied": false,
            "diff": "--- README.md\n+++ README.md\n@@\n-old text\n+new text\n"
        })
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n"
    );
}

#[test]
fn file_edit_apply_writes_replacement() {
    let (temp, sandbox) = sandbox_fixture();
    let tool = FileEditTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "path": "README.md",
                "expected": "old text\n",
                "replacement": "new text\n",
                "apply": true
            }),
        })
        .expect("apply edit");

    assert_eq!(output.value["applied"], true);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "new text\n"
    );
}

#[test]
fn file_edit_rejects_expected_mismatch_without_writing() {
    let (temp, sandbox) = sandbox_fixture();
    let tool = FileEditTool::new(sandbox);

    let error = tool
        .execute(ToolInput {
            value: json!({
                "path": "README.md",
                "expected": "different\n",
                "replacement": "new text\n",
                "apply": true
            }),
        })
        .expect_err("mismatch rejected");

    assert!(error.to_string().contains("expected content did not match"));
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n"
    );
}

#[test]
fn file_edit_rejects_traversal() {
    let (temp, sandbox) = sandbox_fixture();
    fs::write(temp.path().join("secret.txt"), "secret").expect("outside fixture");
    let tool = FileEditTool::new(sandbox);

    let error = tool
        .execute(ToolInput {
            value: json!({
                "path": "../secret.txt",
                "replacement": "new secret\n",
                "apply": false
            }),
        })
        .expect_err("traversal rejected");

    assert!(error.to_string().contains("path escapes workspace"));
}

#[test]
fn file_edit_tool_registers_in_registry() {
    let (_temp, sandbox) = sandbox_fixture();
    let mut registry = ToolRegistry::default();

    registry
        .register(FileEditTool::new(sandbox))
        .expect("register edit tool");

    assert!(
        registry
            .get(&ToolName::new("workspace.edit_file").expect("name"))
            .is_some()
    );
}

#[test]
fn write_file_tool_creates_file_inside_workspace() {
    let (temp, sandbox) = sandbox_fixture();
    let tool = WriteFileTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({
                "path": "notes/todo.txt",
                "content": "ship the real agent\n"
            }),
        })
        .expect("write file");

    assert_eq!(output.value["path"], "notes/todo.txt");
    assert_eq!(output.value["bytes"], json!("ship the real agent\n".len()));
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/notes/todo.txt")).expect("read file"),
        "ship the real agent\n"
    );
}

#[test]
fn write_file_tool_rejects_traversal() {
    let (temp, sandbox) = sandbox_fixture();
    let tool = WriteFileTool::new(sandbox);

    let error = tool
        .execute(ToolInput {
            value: json!({
                "path": "../secret.txt",
                "content": "secret"
            }),
        })
        .expect_err("traversal rejected");

    assert!(error.to_string().contains("path escapes workspace"));
    assert!(!temp.path().join("secret.txt").exists());
}
