use std::fs;

use agent_policy::ApprovalPreviewSummary;
use agent_sandbox::WorkspaceSandbox;
use agent_tool::{FileEditPreview, ToolInput, ToolName, ToolPreview, preview_tool_call};
use serde_json::json;
use tempfile::{TempDir, tempdir};

fn workspace_fixture() -> (TempDir, WorkspaceSandbox) {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(root.join("README.md"), "old text\n").expect("readme");
    fs::create_dir(root.join("subdir")).expect("subdir");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    (temp, sandbox)
}

fn tool_name(name: &str) -> ToolName {
    ToolName::new(name).expect("valid tool name")
}

#[test]
fn file_edit_preview_summarizes_diff_without_mutating_file() {
    let (temp, sandbox) = workspace_fixture();
    let input = ToolInput {
        value: json!({
            "path": "README.md",
            "replacement": "new text\n",
            "apply": true
        }),
    };

    let preview = preview_tool_call(&tool_name("workspace.edit_file"), &input, Some(&sandbox))
        .expect("file edit preview");

    let ToolPreview::FileEdit(FileEditPreview {
        path,
        diff,
        added_lines,
        removed_lines,
        apply,
        ..
    }) = &preview
    else {
        panic!("expected file edit preview");
    };
    assert_eq!(path, "README.md");
    assert_eq!(
        diff,
        "--- README.md\n+++ README.md\n@@\n-old text\n+new text\n"
    );
    assert_eq!((*added_lines, *removed_lines), (1, 1));
    assert!(*apply);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n"
    );

    let summary = preview.to_approval_preview_summary().expect("summary");
    assert_eq!(
        summary,
        ApprovalPreviewSummary::new("file_edit", "README.md", "Edit README.md: +1 -1 lines")
            .expect("expected summary")
            .with_content_hash(summary.content_hash.clone().expect("content hash"))
    );
}

#[test]
fn write_file_preview_summarizes_create_without_mutating_file() {
    let (temp, sandbox) = workspace_fixture();
    let input = ToolInput {
        value: json!({
            "path": "notes/new.txt",
            "content": "new text\n"
        }),
    };

    let preview = preview_tool_call(&tool_name("workspace.write_file"), &input, Some(&sandbox))
        .expect("write file preview");

    let ToolPreview::FileEdit(FileEditPreview {
        path,
        diff,
        added_lines,
        removed_lines,
        apply,
        ..
    }) = &preview
    else {
        panic!("expected file edit preview");
    };
    assert_eq!(path, "notes/new.txt");
    assert_eq!(
        diff,
        "--- notes/new.txt\n+++ notes/new.txt\n@@\n+new text\n"
    );
    assert_eq!((*added_lines, *removed_lines), (1, 0));
    assert!(*apply);
    assert!(!temp.path().join("workspace/notes/new.txt").exists());
}

#[test]
fn shell_command_preview_redacts_environment_values() {
    let (_temp, sandbox) = workspace_fixture();
    let input = ToolInput {
        value: json!({
            "program": "cargo",
            "args": ["test", "-p", "agent-tool"],
            "cwd": "subdir",
            "timeout_ms": 5000,
            "env": {
                "SAFE_FLAG": "1",
                "API_TOKEN": "secret-token"
            }
        }),
    };

    let preview = preview_tool_call(
        &tool_name("workspace.shell_command"),
        &input,
        Some(&sandbox),
    )
    .expect("command preview");

    let ToolPreview::Command(command) = &preview else {
        panic!("expected command preview");
    };
    assert_eq!(command.program, "cargo");
    assert_eq!(command.args, vec!["test", "-p", "agent-tool"]);
    assert_eq!(command.cwd, "subdir");
    assert_eq!(command.timeout_ms, 5000);
    assert_eq!(command.env.len(), 2);
    assert!(command.env.iter().all(|item| item.value == "<redacted>"));
    assert_eq!(
        command.redaction.redacted_fields,
        vec!["env.API_TOKEN".to_string(), "env.SAFE_FLAG".to_string()]
    );

    let serialized = serde_json::to_string(&preview).expect("serialize preview");
    assert!(!serialized.contains("secret-token"));
}

#[test]
fn git_and_extension_previews_are_typed_and_redacted() {
    let (_temp, sandbox) = workspace_fixture();
    let git_input = ToolInput {
        value: json!({
            "cwd": ".",
            "path": "README.md"
        }),
    };
    let git_preview =
        preview_tool_call(&tool_name("workspace.git_diff"), &git_input, Some(&sandbox))
            .expect("git preview");

    let ToolPreview::Git(git) = git_preview else {
        panic!("expected git preview");
    };
    assert_eq!(git.operation, "diff");
    assert_eq!(git.path.as_deref(), Some("README.md"));

    let plugin_input = ToolInput {
        value: json!({
            "plugin_id": "formatter",
            "tool": "format",
            "api_key": "secret",
            "path": "src/lib.rs"
        }),
    };
    let plugin_preview =
        preview_tool_call(&tool_name("plugin.formatter.format"), &plugin_input, None)
            .expect("plugin preview");

    let ToolPreview::Plugin(plugin) = &plugin_preview else {
        panic!("expected plugin preview");
    };
    assert_eq!(plugin.plugin_id, "formatter");
    assert_eq!(plugin.tool_or_hook, "format");
    assert!(plugin.input_hash.starts_with("fnv1a64:"));
    assert_eq!(
        plugin.redaction.redacted_fields,
        vec!["api_key".to_string()]
    );
    assert!(!plugin.input_summary.contains("secret"));
}

#[test]
fn generic_preview_hashes_unknown_inputs_and_truncates_large_summaries() {
    let long_value = "x".repeat(600);
    let input = ToolInput {
        value: json!({
            "query": long_value,
            "password": "secret"
        }),
    };

    let preview = preview_tool_call(&tool_name("workspace.unknown_tool"), &input, None)
        .expect("generic preview");

    let ToolPreview::Generic(generic) = &preview else {
        panic!("expected generic preview");
    };
    assert!(generic.input_hash.starts_with("fnv1a64:"));
    assert!(generic.redaction.truncated);
    assert_eq!(
        generic.redaction.redacted_fields,
        vec!["password".to_string()]
    );
    assert!(!generic.input_summary.contains("secret"));

    let summary = preview.to_approval_preview_summary().expect("summary");
    assert_eq!(summary.kind, "generic");
    assert!(summary.truncated);
    assert!(summary.redacted);
}

#[test]
fn previews_reject_workspace_path_escape() {
    let (temp, sandbox) = workspace_fixture();
    fs::write(temp.path().join("secret.txt"), "secret").expect("outside file");
    let input = ToolInput {
        value: json!({
            "path": "../secret.txt",
            "replacement": "new secret\n"
        }),
    };

    let error = preview_tool_call(&tool_name("workspace.edit_file"), &input, Some(&sandbox))
        .expect_err("escape rejected");

    assert!(error.to_string().contains("path escapes workspace"));
}
