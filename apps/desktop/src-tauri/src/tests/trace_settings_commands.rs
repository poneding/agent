use agent_core::TraceEntry;
use agent_desktop::DesktopSettings;
use agent_types::ConversationId;
use agent_ui_bridge::{
    UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot, UiModelProviderCapabilities,
    UiModelProviderSummary,
};

fn conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000000021"
        .parse()
        .expect("valid conversation id")
}

fn entry(seq: u64, kind: &str, summary: &str) -> TraceEntry {
    TraceEntry {
        seq,
        kind: kind.to_string(),
        summary: summary.to_string(),
    }
}

fn provider() -> UiModelProviderSummary {
    UiModelProviderSummary::new(
        "fake",
        "fake",
        "fake-scripted",
        true,
        UiModelProviderCapabilities::new(true, false, false, false, Some(4096), Some(1024), None),
    )
    .expect("valid provider")
}

fn config_snapshot(message: &str) -> UiConfigSnapshot {
    UiConfigSnapshot::new(
        "fake",
        "fake-scripted",
        vec![provider()],
        vec![
            UiConfigDiagnostic::new(
                UiConfigDiagnosticSeverity::Warning,
                "config.fixture",
                message,
                Some("model.provider".to_string()),
                Some("test".to_string()),
            )
            .expect("valid diagnostic"),
        ],
    )
    .expect("valid config snapshot")
}

#[test]
fn trace_command_returns_empty_timeline() {
    let timeline =
        crate::trace_commands::build_trace_response(conversation_id().to_string(), Vec::new())
            .expect("empty trace is valid");

    assert_eq!(timeline.conversation_id, conversation_id());
    assert!(timeline.items.is_empty());
}

#[test]
fn trace_command_returns_populated_redacted_timeline() {
    let timeline = crate::trace_commands::build_trace_response(
        conversation_id().to_string(),
        vec![
            entry(2, "tool_error", "failed with token sk-live"),
            entry(1, "checkpoint", "saved workspace"),
        ],
    )
    .expect("trace is valid");

    assert_eq!(timeline.items[0].seq, 1);
    assert_eq!(
        timeline.items[1].severity,
        agent_desktop::TraceTimelineSeverity::Error
    );
    assert_eq!(timeline.logs[0].sequence, 1);
    assert_eq!(timeline.logs[1].source, "tool_error");
    assert_eq!(
        timeline.logs[1].severity,
        agent_desktop::DesktopRuntimeLogSeverity::Error
    );
    let serialized = serde_json::to_string(&timeline).expect("serialize trace");
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-live"));
    assert!(!serialized.contains("token"));
}

#[test]
fn trace_command_maps_duplicate_sequence_to_safe_error() {
    let error = crate::trace_commands::build_trace_response(
        conversation_id().to_string(),
        vec![
            entry(1, "tool_call", "input token sk-secret"),
            entry(1, "final", "done"),
        ],
    )
    .expect_err("duplicate sequence rejected");

    assert_eq!(error.code, "desktop.trace.invalid");
    assert!(error.message.contains("duplicate trace sequence"));
    assert!(!error.message.contains("sk-secret"));
}

#[test]
fn config_snapshot_command_returns_safe_snapshot() {
    let snapshot = crate::settings_commands::build_config_snapshot_response(config_snapshot(
        "using fake provider",
    ))
    .expect("snapshot is valid");

    assert_eq!(snapshot.snapshot.selected_provider, "fake");
    assert_eq!(snapshot.snapshot.selected_model, "fake-scripted");
    assert_eq!(snapshot.snapshot.providers[0].id, "fake");
}

#[test]
fn validate_settings_returns_diagnostics_without_mutation() {
    let mut settings = DesktopSettings::default();
    settings.model.provider = " ".to_string();

    let diagnostics = crate::settings_commands::validate_settings_request(settings)
        .expect("diagnostics returned");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, "error");
    assert_eq!(diagnostics[0].code, "desktop.settings.invalid");
    assert!(
        diagnostics[0]
            .message
            .contains("model provider cannot be empty")
    );
}

#[test]
fn settings_diagnostics_are_redacted() {
    let mut settings = DesktopSettings::default();
    settings.policy.default_decision = "token sk-secret password".to_string();

    let diagnostics = crate::settings_commands::validate_settings_request(settings)
        .expect("diagnostics returned");
    let message = &diagnostics[0].message;

    assert!(message.contains("[redacted]"));
    assert!(!message.contains("sk-secret"));
    assert!(!message.contains("token"));
    assert!(!message.contains("password"));
}

#[test]
fn config_snapshot_diagnostics_are_redacted() {
    let snapshot = crate::settings_commands::build_config_snapshot_response(config_snapshot(
        "api_key sk-secret token password should not leak",
    ))
    .expect("snapshot is valid");
    let message = &snapshot.snapshot.diagnostics[0].message;

    assert_eq!(
        message,
        "[redacted] [redacted] [redacted] [redacted] should not leak"
    );
}
