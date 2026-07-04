use std::fs;

use agent_sandbox::WorkspaceSandbox;
use agent_tool::builtins::{ReadFileTool, SearchTextTool};
use agent_tool::{Tool, ToolInput, ToolName, ToolRegistry};
use serde_json::json;
use tempfile::tempdir;

fn sandbox_fixture() -> (tempfile::TempDir, WorkspaceSandbox) {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(root.join("README.md"), "hello from readme\n").expect("readme");
    fs::create_dir(root.join("src")).expect("src dir");
    fs::write(
        root.join("src/lib.rs"),
        "pub struct RuntimeEvent;\nfn helper() {}\n",
    )
    .expect("source");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    (temp, sandbox)
}

#[test]
fn read_file_tool_returns_file_content() {
    let (_temp, sandbox) = sandbox_fixture();
    let tool = ReadFileTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({ "path": "README.md" }),
        })
        .expect("read file");

    assert_eq!(
        output.value,
        json!({
            "path": "README.md",
            "content": "hello from readme\n"
        })
    );
}

#[test]
fn search_text_tool_returns_matching_lines() {
    let (_temp, sandbox) = sandbox_fixture();
    let tool = SearchTextTool::new(sandbox);

    let output = tool
        .execute(ToolInput {
            value: json!({ "path": "src", "query": "RuntimeEvent" }),
        })
        .expect("search text");

    assert_eq!(
        output.value,
        json!({
            "matches": [
                {
                    "path": "src/lib.rs",
                    "line": 1,
                    "text": "pub struct RuntimeEvent;"
                }
            ]
        })
    );
}

#[test]
fn read_only_tools_reject_traversal() {
    let (temp, sandbox) = sandbox_fixture();
    fs::write(temp.path().join("secret.txt"), "secret").expect("outside fixture");
    let tool = ReadFileTool::new(sandbox);

    let error = tool
        .execute(ToolInput {
            value: json!({ "path": "../secret.txt" }),
        })
        .expect_err("traversal rejected");

    assert!(error.to_string().contains("path escapes workspace"));
}

#[test]
fn read_only_builtins_register_in_registry() {
    let (_temp, sandbox) = sandbox_fixture();
    let mut registry = ToolRegistry::default();

    registry
        .register(ReadFileTool::new(sandbox.clone()))
        .expect("register read file");
    registry
        .register(SearchTextTool::new(sandbox))
        .expect("register search text");

    assert!(
        registry
            .get(&ToolName::new("workspace.read_file").expect("name"))
            .is_some()
    );
    assert!(
        registry
            .get(&ToolName::new("workspace.search_text").expect("name"))
            .is_some()
    );
}
