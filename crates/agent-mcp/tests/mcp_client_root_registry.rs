use agent_mcp::{McpClientRootCandidate, McpClientRootInspectionSeverity, McpClientRootRegistry};
use tempfile::TempDir;

#[test]
fn mcp_client_root_registry_registers_valid_roots_in_order() {
    let workspace = TempDir::new().expect("workspace");
    let alpha = workspace.path().join("alpha");
    let beta = workspace.path().join("beta");
    std::fs::create_dir_all(&alpha).expect("alpha dir");
    std::fs::create_dir_all(&beta).expect("beta dir");

    let mut registry =
        McpClientRootRegistry::with_workspace_root(workspace.path()).expect("registry");
    registry
        .register(McpClientRootCandidate::new(&beta, Some("Beta")))
        .expect("register beta");
    registry
        .register(McpClientRootCandidate::new(&alpha, Some("Alpha")))
        .expect("register alpha");

    let roots = registry.list_result();
    assert_eq!(roots.roots.len(), 2);
    assert_eq!(roots.roots[0].name.as_deref(), Some("Alpha"));
    assert_eq!(roots.roots[1].name.as_deref(), Some("Beta"));
    assert!(roots.roots[0].uri.starts_with("file://"));
    assert!(roots.roots[0].uri.ends_with("/alpha"));
    assert!(!registry.is_empty());

    let inspection = registry.inspection();
    assert_eq!(inspection.roots.len(), 2);
    assert!(inspection.diagnostics.is_empty());
}

#[test]
fn mcp_client_root_registry_handles_duplicate_and_disabled_roots() {
    let workspace = TempDir::new().expect("workspace");
    let active = workspace.path().join("active");
    let disabled = workspace.path().join("disabled");
    std::fs::create_dir_all(&active).expect("active dir");
    std::fs::create_dir_all(&disabled).expect("disabled dir");

    let mut registry =
        McpClientRootRegistry::with_workspace_root(workspace.path()).expect("registry");
    registry
        .register(McpClientRootCandidate::new(&active, Some("Active")))
        .expect("active");
    registry
        .register(McpClientRootCandidate::new(&active, Some("Active")))
        .expect("duplicate is idempotent");
    registry
        .register(McpClientRootCandidate::disabled(
            &disabled,
            Some("Disabled"),
        ))
        .expect("disabled");

    let roots = registry.list_result();
    assert_eq!(roots.roots.len(), 1);
    assert_eq!(roots.roots[0].name.as_deref(), Some("Active"));

    let inspection = registry.inspection();
    assert_eq!(inspection.roots.len(), 2);
    assert!(inspection.roots.iter().any(|root| !root.enabled));
    assert_eq!(inspection.diagnostics.len(), 1);
    assert_eq!(
        inspection.diagnostics[0].severity,
        McpClientRootInspectionSeverity::Warning
    );
    assert_eq!(inspection.diagnostics[0].code, "disabled_root");

    let duplicate = registry
        .register(McpClientRootCandidate::new(&active, Some("Renamed")))
        .expect_err("conflicting duplicate");
    assert!(duplicate.to_string().contains("duplicate MCP client root"));
}

#[test]
fn mcp_client_root_registry_reports_empty_when_only_disabled_roots_exist() {
    let workspace = TempDir::new().expect("workspace");
    let disabled = workspace.path().join("disabled");
    std::fs::create_dir_all(&disabled).expect("disabled dir");

    let mut registry =
        McpClientRootRegistry::with_workspace_root(workspace.path()).expect("registry");
    registry
        .register(McpClientRootCandidate::disabled(
            &disabled,
            Some("Disabled"),
        ))
        .expect("disabled");

    assert!(registry.is_empty());
    assert!(registry.list_result().roots.is_empty());
    assert_eq!(registry.inspection().roots.len(), 1);
}

#[test]
fn mcp_client_root_registry_rejects_invalid_paths_safely() {
    let workspace = TempDir::new().expect("workspace");
    let outside = TempDir::new().expect("outside");
    let missing = workspace.path().join("missing");
    let secret = workspace.path().join("sk-secret");
    std::fs::create_dir_all(&secret).expect("secret-like dir");

    let mut registry =
        McpClientRootRegistry::with_workspace_root(workspace.path()).expect("registry");

    let outside_error = registry
        .register(McpClientRootCandidate::new(outside.path(), Some("Outside")))
        .expect_err("outside root");
    assert!(
        outside_error
            .to_string()
            .contains("outside configured workspace")
    );

    let traversal_error = registry
        .register(McpClientRootCandidate::new(
            workspace.path().join(".."),
            Some("Parent"),
        ))
        .expect_err("traversal root");
    assert!(
        traversal_error
            .to_string()
            .contains("outside configured workspace")
    );

    let missing_error = registry
        .register(McpClientRootCandidate::new(&missing, Some("Missing")))
        .expect_err("missing root");
    assert!(
        missing_error
            .to_string()
            .contains("invalid MCP client root path")
    );

    let secret_path_error = registry
        .register(McpClientRootCandidate::new(&secret, Some("Secret")))
        .expect_err("secret path");
    assert!(secret_path_error.to_string().contains("[redacted]"));
    assert!(!secret_path_error.to_string().contains("sk-secret"));

    let secret_name_error = registry
        .register(McpClientRootCandidate::new(
            workspace.path(),
            Some("api_key"),
        ))
        .expect_err("secret name");
    assert!(secret_name_error.to_string().contains("[redacted]"));
    assert!(!secret_name_error.to_string().contains("api_key"));
}
