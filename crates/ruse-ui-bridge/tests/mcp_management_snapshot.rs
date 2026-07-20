use ruse_ui_bridge::{
    UiMcpManagementDiagnostic, UiMcpManagementSnapshot, UiMcpServerManagementSummary,
};

#[test]
fn ui_bridge_mcp_management_snapshot_serializes_stable_shape() {
    let snapshot = UiMcpManagementSnapshot::new(vec![
        UiMcpServerManagementSummary::new(
            "stdio-server",
            "Stdio Server",
            true,
            "healthy",
            "stdio",
            1,
            true,
            false,
            false,
            "not_applicable",
            "not_applicable",
            "healthy",
            vec![
                UiMcpManagementDiagnostic::new("info", "server_health", "healthy")
                    .expect("diagnostic"),
            ],
        )
        .expect("summary"),
    ])
    .expect("snapshot");

    let value = serde_json::to_value(&snapshot).expect("snapshot json");

    assert_eq!(value["servers"][0]["server_id"], "stdio-server");
    assert_eq!(value["servers"][0]["transport_kind"], "stdio");
    assert_eq!(value["servers"][0]["root_count"], 1);
    assert_eq!(value["servers"][0]["sampling_available"], true);
    assert_eq!(
        value["servers"][0]["diagnostics"][0]["code"],
        "server_health"
    );
}

#[test]
fn ui_bridge_mcp_management_snapshot_validates_required_fields() {
    let error = UiMcpServerManagementSummary::new(
        "",
        "Display",
        true,
        "healthy",
        "stdio",
        0,
        false,
        false,
        false,
        "not_applicable",
        "not_applicable",
        "unknown",
        Vec::new(),
    )
    .expect_err("server id required");

    assert!(error.to_string().contains("MCP server id"));
}
