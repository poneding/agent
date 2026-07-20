use ruse_mcp::{McpResourceExportRegistry, McpResourceReadResult};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_types::{ActorRef, CorrelationId};
use tempfile::TempDir;

fn allow_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    )
}

fn deny_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Deny {
            reason: "fixture deny".to_string(),
        },
    )
}

#[test]
fn mcp_server_resource_template_matching_extracts_variables() {
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
        )
        .expect("template");

    let matches = registry.match_resource_templates("agent://workspace/file/src/lib.rs");

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].uri_template, "agent://workspace/file/{path}");
    assert_eq!(matches[0].uri, "agent://workspace/file/src/lib.rs");
    assert_eq!(
        matches[0].variables.get("path").map(String::as_str),
        Some("src/lib.rs")
    );
}

#[test]
fn mcp_server_resource_template_matching_reads_file_template() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(temp.path().join("src")).expect("src dir");
    std::fs::write(temp.path().join("src/lib.rs"), "hello template").expect("fixture");
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_file_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            Some("Read workspace file"),
            Some("text/plain"),
            temp.path(),
            "{path}",
        )
        .expect("template");

    let output = registry
        .read_resource(
            &allow_policy("agent://workspace/file/{path}"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/src/lib.rs",
        )
        .expect("read");

    assert_eq!(
        output,
        McpResourceReadResult {
            uri: "agent://workspace/file/src/lib.rs".to_string(),
            contents: "hello template".to_string(),
            mime_type: Some("text/plain".to_string()),
            truncated: false,
        }
    );
}

#[test]
fn mcp_server_resource_template_matching_static_resource_wins_before_template() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "dynamic").expect("fixture");
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_file_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
            temp.path(),
            "{path}",
        )
        .expect("template");
    registry
        .export_resource(
            "agent://workspace/file/README.md",
            "Static README",
            None::<String>,
            "static",
        )
        .expect("static");

    let output = registry
        .read_resource(
            &allow_policy("agent://workspace/file/README.md"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/README.md",
        )
        .expect("read");

    assert_eq!(output.contents, "static");
}

#[test]
fn mcp_server_resource_template_matching_rejects_ambiguous_matches() {
    let temp = TempDir::new().expect("tempdir");
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_file_resource_template(
            "agent://workspace/{path}",
            "Workspace path",
            None::<String>,
            None::<String>,
            temp.path(),
            "{path}",
        )
        .expect("broad template");
    registry
        .export_file_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
            temp.path(),
            "{path}",
        )
        .expect("file template");

    let error = registry
        .read_resource(
            &allow_policy("agent://workspace/file/README.md"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/README.md",
        )
        .expect_err("ambiguous");

    assert!(
        error
            .to_string()
            .contains("ambiguous MCP resource template")
    );
}

#[test]
fn mcp_server_resource_template_matching_rejects_path_traversal_without_leaking_contents() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("public.txt"), "public").expect("public");
    let outside = temp
        .path()
        .parent()
        .expect("parent")
        .join("mcp-outside.txt");
    std::fs::write(&outside, "api_key sk-secret").expect("secret");
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_file_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
            temp.path(),
            "{path}",
        )
        .expect("template");

    let error = registry
        .read_resource(
            &allow_policy("agent://workspace/file/../mcp-outside.txt"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/../mcp-outside.txt",
        )
        .expect_err("traversal");

    let rendered = error.to_string();
    assert!(rendered.contains("invalid MCP resource template path"));
    assert!(!rendered.contains("api_key"));
    let _ = std::fs::remove_file(outside);
}

#[test]
fn mcp_server_resource_template_matching_denied_read_returns_no_content() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "private template content").expect("fixture");
    let mut registry = McpResourceExportRegistry::default();
    registry
        .export_file_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
            temp.path(),
            "{path}",
        )
        .expect("template");

    let error = registry
        .read_resource(
            &deny_policy("agent://workspace/file/{path}"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/README.md",
        )
        .expect_err("denied");

    assert!(
        error
            .to_string()
            .contains("policy denied MCP resource export")
    );
    assert!(!error.to_string().contains("private template content"));
}

#[test]
fn mcp_server_resource_template_matching_bounds_file_contents() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "hello template world").expect("fixture");
    let mut registry = McpResourceExportRegistry::default().with_max_content_bytes(8);
    registry
        .export_file_resource_template(
            "agent://workspace/file/{path}",
            "Workspace file",
            None::<String>,
            None::<String>,
            temp.path(),
            "{path}",
        )
        .expect("template");

    let output = registry
        .read_resource(
            &allow_policy("agent://workspace/file/{path}"),
            ActorRef::system(),
            CorrelationId::new(),
            "agent://workspace/file/README.md",
        )
        .expect("read");

    assert_eq!(output.contents, "hello te");
    assert!(output.truncated);
}
