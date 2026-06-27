use crate::bootstrap::{DesktopBootstrapDiagnostic, DesktopRuntimeState, build_desktop_bootstrap};
use agent_desktop::DesktopRoute;

#[test]
fn bootstrap_starts_in_not_started_conversation_route() {
    let bootstrap = build_desktop_bootstrap("agent", "0.1.0", Vec::new()).expect("valid bootstrap");

    assert_eq!(bootstrap.app_name, "agent");
    assert_eq!(bootstrap.app_version, "0.1.0");
    assert_eq!(bootstrap.runtime_state, DesktopRuntimeState::NotStarted);
    assert_eq!(bootstrap.route, DesktopRoute::Conversation);
    assert!(bootstrap.diagnostics.is_empty());
}

#[test]
fn bootstrap_redacts_secret_like_diagnostics() {
    let diagnostic = DesktopBootstrapDiagnostic::new(
        "warning",
        "config.secret",
        "api_key sk-test token password should not leak",
    )
    .expect("valid diagnostic");
    let bootstrap =
        build_desktop_bootstrap("agent", "0.1.0", vec![diagnostic]).expect("valid bootstrap");

    assert_eq!(bootstrap.diagnostics.len(), 1);
    assert_eq!(
        bootstrap.diagnostics[0].message,
        "[redacted] [redacted] [redacted] [redacted] should not leak"
    );
}

#[test]
fn bootstrap_rejects_empty_identity_fields() {
    assert!(build_desktop_bootstrap("", "0.1.0", Vec::new()).is_err());
    assert!(build_desktop_bootstrap("agent", " ", Vec::new()).is_err());
}
