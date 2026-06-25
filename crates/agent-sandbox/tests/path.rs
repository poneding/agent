use std::fs;

use agent_sandbox::{SandboxError, WorkspaceSandbox};
use tempfile::tempdir;

#[test]
fn sandbox_resolves_relative_path_inside_root() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(root.join("README.md"), "hello").expect("fixture");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");

    let resolved = sandbox
        .resolve_read_path("README.md")
        .expect("resolve path");

    assert_eq!(
        resolved,
        root.join("README.md").canonicalize().expect("canonical")
    );
}

#[test]
fn sandbox_allows_absolute_path_inside_root() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    let file = root.join("README.md");
    fs::write(&file, "hello").expect("fixture");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");

    let resolved = sandbox
        .resolve_read_path(file.to_str().expect("utf8 path"))
        .expect("resolve path");

    assert_eq!(resolved, file.canonicalize().expect("canonical"));
}

#[test]
fn sandbox_rejects_parent_traversal_outside_root() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(temp.path().join("secret.txt"), "secret").expect("outside fixture");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");

    let error = sandbox
        .resolve_read_path("../secret.txt")
        .expect_err("escape rejected");

    assert!(matches!(error, SandboxError::PathEscape(_)));
}

#[cfg(unix)]
#[test]
fn sandbox_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace");
    fs::write(temp.path().join("secret.txt"), "secret").expect("outside fixture");
    symlink(temp.path().join("secret.txt"), root.join("secret-link")).expect("symlink");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");

    let error = sandbox
        .resolve_read_path("secret-link")
        .expect_err("symlink escape rejected");

    assert!(matches!(error, SandboxError::PathEscape(_)));
}
