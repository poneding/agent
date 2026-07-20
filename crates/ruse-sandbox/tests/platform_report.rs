use ruse_sandbox::{
    CapabilityStatus, PlatformCapabilityReport, PlatformReportError, PluginIsolationCapability,
    SandboxCapability, SecretProviderCapability,
};
use serde_json::json;

#[test]
fn platform_report_serializes_stable_shape() {
    let report = PlatformCapabilityReport::new(
        "test-os",
        SandboxCapability::supported("workspace path confinement"),
        SandboxCapability::supported("canonical symlink checks"),
        SandboxCapability::supported("child process execution"),
        SandboxCapability::unknown("network policy supplied by host"),
        PluginIsolationCapability::new(
            SandboxCapability::supported("process plugins"),
            SandboxCapability::unsupported("wasm runtime unavailable"),
        ),
        SecretProviderCapability::unavailable("no local provider configured"),
    )
    .expect("valid report");

    assert_eq!(
        serde_json::to_value(report).expect("serialize report"),
        json!({
            "os": "test-os",
            "filesystem_sandbox": {
                "status": "supported",
                "detail": "workspace path confinement"
            },
            "symlink_protection": {
                "status": "supported",
                "detail": "canonical symlink checks"
            },
            "process_execution": {
                "status": "supported",
                "detail": "child process execution"
            },
            "network_default": {
                "status": "unknown",
                "detail": "network policy supplied by host"
            },
            "plugin_isolation": {
                "process": {
                    "status": "supported",
                    "detail": "process plugins"
                },
                "wasm": {
                    "status": "unsupported",
                    "detail": "wasm runtime unavailable"
                }
            },
            "secret_provider": {
                "status": "unsupported",
                "provider": null,
                "detail": "no local provider configured"
            }
        })
    );
}

#[test]
fn platform_report_builder_rejects_empty_os() {
    let error = PlatformCapabilityReport::new(
        " ",
        SandboxCapability::unknown("filesystem"),
        SandboxCapability::unknown("symlink"),
        SandboxCapability::unknown("process"),
        SandboxCapability::unknown("network"),
        PluginIsolationCapability::unknown(),
        SecretProviderCapability::unknown("secret provider unknown"),
    )
    .expect_err("empty os rejected");

    assert!(matches!(error, PlatformReportError::EmptyOs));
}

#[test]
fn platform_report_default_includes_current_os() {
    let report = PlatformCapabilityReport::current().expect("current report");

    assert_eq!(report.os, std::env::consts::OS);
    assert!(!report.os.trim().is_empty());
}

#[test]
fn platform_report_represents_unsupported_capabilities() {
    let unsupported = SandboxCapability::unsupported("not available");

    assert_eq!(unsupported.status, CapabilityStatus::Unsupported);
    assert_eq!(unsupported.detail, "not available");
}

#[test]
fn platform_report_includes_plugin_isolation_capabilities() {
    let report = PlatformCapabilityReport::new(
        "test-os",
        SandboxCapability::supported("filesystem"),
        SandboxCapability::supported("symlink"),
        SandboxCapability::supported("process"),
        SandboxCapability::unknown("network"),
        PluginIsolationCapability::new(
            SandboxCapability::supported("process isolation"),
            SandboxCapability::supported("wasm validation"),
        ),
        SecretProviderCapability::unknown("secret provider unknown"),
    )
    .expect("valid report");

    assert_eq!(
        report.plugin_isolation.process.status,
        CapabilityStatus::Supported
    );
    assert_eq!(
        report.plugin_isolation.wasm.status,
        CapabilityStatus::Supported
    );
}
