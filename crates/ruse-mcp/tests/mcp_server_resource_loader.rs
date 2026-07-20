use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ruse_mcp::{
    McpError, McpServerProfileRuntime, build_mcp_server_resource_profile_from_manifest,
    parse_mcp_server_export_manifest_json,
};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_types::{ActorRef, CorrelationId};

fn temp_root(test_name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ruse-mcp-{test_name}-{unique}"));
    fs::create_dir_all(&root).expect("temp root");
    root
}

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent");
    }
    fs::write(path, contents).expect("write fixture");
}

fn allow_read_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    )
}

fn expect_loader_error(result: Result<McpServerProfileRuntime, McpError>) -> McpError {
    match result {
        Ok(_) => panic!("expected loader error"),
        Err(error) => error,
    }
}

#[test]
fn mcp_server_resource_loader_builds_visible_file_resource_profile() {
    let root = temp_root("visible-resource");
    write_file(&root, "README.md", "hello resource");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://readme",
                  "name": "README",
                  "path": "README.md",
                  "mime_type": "text/markdown"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_resource_profile_from_manifest(&manifest, None, &root)
        .expect("build profile");
    let listed = runtime.resource_exports().list_resources();
    let read = runtime
        .resource_exports()
        .read_resource(
            &allow_read_policy("agent://readme"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://readme",
        )
        .expect("read resource");

    assert_eq!(runtime.profile().name, "default");
    assert_eq!(runtime.profile().resource_uris, ["agent://readme"]);
    assert_eq!(listed[0].name, "README");
    assert_eq!(listed[0].mime_type, Some("text/markdown".to_string()));
    assert_eq!(read.contents, "hello resource");
}

#[test]
fn mcp_server_resource_loader_hides_disabled_resources() {
    let root = temp_root("disabled-resource");
    write_file(&root, "hidden.txt", "hidden");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://hidden",
                  "name": "Hidden",
                  "path": "hidden.txt",
                  "enabled": false
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_resource_profile_from_manifest(&manifest, None, &root)
        .expect("build profile");

    assert!(runtime.profile().resource_uris.is_empty());
    assert!(runtime.resource_exports().list_resources().is_empty());
}

#[test]
fn mcp_server_resource_loader_fails_before_startup_for_missing_files() {
    let root = temp_root("missing-resource");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://missing",
                  "name": "Missing",
                  "path": "missing.txt"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let error = expect_loader_error(build_mcp_server_resource_profile_from_manifest(
        &manifest, None, &root,
    ));

    assert!(error.to_string().contains("failed to read"));
    assert!(!error.to_string().contains("hello resource"));
}

#[test]
fn mcp_server_resource_loader_requires_sensitive_flag_for_secret_like_contents() {
    let root = temp_root("sensitive-resource");
    write_file(&root, "private.txt", "api_key sk-secret");
    let non_sensitive = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://private",
                  "name": "Private",
                  "path": "private.txt"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");
    let sensitive = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://private",
                  "name": "Private",
                  "path": "private.txt",
                  "sensitive": true
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let error = expect_loader_error(build_mcp_server_resource_profile_from_manifest(
        &non_sensitive,
        None,
        &root,
    ));
    let loaded = build_mcp_server_resource_profile_from_manifest(&sensitive, None, &root)
        .expect("sensitive resource loads");

    assert!(
        error
            .to_string()
            .contains("requires explicit sensitive export")
    );
    assert!(!error.to_string().contains("sk-secret"));
    assert_eq!(loaded.profile().resource_uris, ["agent://private"]);
}

#[test]
fn mcp_server_resource_loader_rejects_paths_outside_manifest_root() {
    let root = temp_root("path-escape");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://outside",
                  "name": "Outside",
                  "path": "../outside.txt"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let error = expect_loader_error(build_mcp_server_resource_profile_from_manifest(
        &manifest, None, &root,
    ));

    assert!(
        error
            .to_string()
            .contains("must stay under manifest directory")
    );
}
