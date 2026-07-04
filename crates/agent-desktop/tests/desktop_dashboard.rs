use agent_desktop::{
    DesktopConfigSnapshot, DesktopConversationItem, DesktopDashboardDiagnostic,
    DesktopDashboardDiagnosticSeverity, DesktopDashboardInput, DesktopError,
    DesktopSessionReadiness, DesktopSessionReadinessState, DesktopTaskItem,
    build_dashboard_snapshot, build_workspace_snapshot,
};
use agent_types::{ConversationId, SessionId, TaskId, TaskState, Timestamp, WorkspaceId};
use agent_ui_bridge::{UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-07-01T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn workspace_id() -> WorkspaceId {
    id("00000000-0000-0000-0000-000000000001")
}

fn conversation_id() -> ConversationId {
    id("00000000-0000-0000-0000-000000000002")
}

fn session_id() -> SessionId {
    id("00000000-0000-0000-0000-000000000003")
}

fn task_id(value: u32) -> TaskId {
    id(&format!("00000000-0000-0000-0000-{value:012}"))
}

fn task(value: u32, state: TaskState, summary: &str) -> DesktopTaskItem {
    DesktopTaskItem::new(
        task_id(value),
        workspace_id(),
        Some(conversation_id()),
        state,
        summary,
        ts(value as u8),
    )
    .expect("valid task")
}

fn config_snapshot(
    selected_provider: &str,
    selected_model: &str,
    enabled: bool,
    diagnostic_count: usize,
) -> DesktopConfigSnapshot {
    let providers = vec![
        agent_ui_bridge::UiModelProviderSummary::new(
            selected_provider,
            "fake",
            selected_model,
            enabled,
            agent_ui_bridge::UiModelProviderCapabilities::new(
                true,
                true,
                false,
                false,
                Some(8_192),
                Some(1_024),
                None,
            ),
        )
        .expect("valid provider"),
    ];
    let diagnostics = (0..diagnostic_count)
        .map(|index| {
            UiConfigDiagnostic::new(
                UiConfigDiagnosticSeverity::Warning,
                format!("provider.warning.{index}"),
                format!("provider warning {index}"),
                None,
                None,
            )
            .expect("valid diagnostic")
        })
        .collect::<Vec<_>>();
    DesktopConfigSnapshot::new(
        UiConfigSnapshot::new(selected_provider, selected_model, providers, diagnostics)
            .expect("valid config"),
    )
    .expect("valid desktop config")
}

fn workspace_snapshot(tasks: Vec<DesktopTaskItem>) -> agent_desktop::DesktopWorkspaceSnapshot {
    build_workspace_snapshot(
        workspace_id(),
        Some(conversation_id()),
        vec![
            DesktopConversationItem::new(conversation_id(), "Main", ts(1), 0)
                .expect("valid conversation"),
        ],
        tasks,
    )
    .expect("valid workspace snapshot")
}

fn connected_input(tasks: Vec<DesktopTaskItem>) -> DesktopDashboardInput {
    DesktopDashboardInput {
        workspace: workspace_snapshot(tasks),
        workspace_root_display: Some("/workspace/agent".to_string()),
        session: DesktopSessionReadiness {
            state: DesktopSessionReadinessState::Connected,
            session_id: Some(session_id()),
            conversation_id: Some(conversation_id()),
            pending_frame_count: 2,
        },
        config: config_snapshot("fake", "fake-scripted", true, 1),
        pending_approval_count: 3,
        diagnostics: Vec::new(),
    }
}

#[test]
fn dashboard_serializes_connected_shape() {
    let snapshot = build_dashboard_snapshot(connected_input(vec![task(
        10,
        TaskState::Running,
        "run verification",
    )]))
    .expect("dashboard builds");

    let value = serde_json::to_value(snapshot).expect("serialize dashboard");

    assert_eq!(
        value["workspace_id"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(value["workspace_root_display"], "/workspace/agent");
    assert_eq!(value["session"]["state"], "connected");
    assert_eq!(
        value["session"]["session_id"],
        "00000000-0000-0000-0000-000000000003"
    );
    assert_eq!(value["session"]["pending_frame_count"], 2);
    assert_eq!(value["provider"]["provider_id"], "fake");
    assert_eq!(value["provider"]["model_id"], "fake-scripted");
    assert_eq!(value["provider"]["enabled"], true);
    assert_eq!(value["provider"]["diagnostic_count"], 1);
    assert_eq!(value["pending_approval_count"], 3);
}

#[test]
fn dashboard_counts_task_states_from_typed_states() {
    let snapshot = build_dashboard_snapshot(connected_input(vec![
        task(1, TaskState::Queued, "queued"),
        task(2, TaskState::Running, "running"),
        task(3, TaskState::WaitingApproval, "needs approval"),
        task(4, TaskState::Failed, "failed"),
        task(5, TaskState::Succeeded, "done"),
    ]))
    .expect("dashboard builds");

    assert_eq!(snapshot.task_counts.total, 5);
    assert_eq!(snapshot.task_counts.active, 2);
    assert_eq!(snapshot.task_counts.waiting_approval, 1);
    assert_eq!(snapshot.task_counts.failed, 1);
    assert_eq!(snapshot.task_counts.terminal, 2);
}

#[test]
fn dashboard_supports_empty_not_started_state() {
    let input = DesktopDashboardInput {
        workspace: build_workspace_snapshot(workspace_id(), None, Vec::new(), Vec::new())
            .expect("empty workspace"),
        workspace_root_display: None,
        session: DesktopSessionReadiness {
            state: DesktopSessionReadinessState::NotStarted,
            session_id: None,
            conversation_id: None,
            pending_frame_count: 0,
        },
        config: config_snapshot("fake", "fake-scripted", false, 0),
        pending_approval_count: 0,
        diagnostics: Vec::new(),
    };

    let snapshot = build_dashboard_snapshot(input).expect("dashboard builds");

    assert_eq!(
        snapshot.session.state,
        DesktopSessionReadinessState::NotStarted
    );
    assert_eq!(snapshot.session.session_id, None);
    assert_eq!(snapshot.session.conversation_id, None);
    assert_eq!(snapshot.task_counts.total, 0);
    assert!(!snapshot.provider.enabled);
}

#[test]
fn dashboard_bounds_and_redacts_diagnostics_and_display_paths() {
    let mut input = connected_input(Vec::new());
    input.workspace_root_display = Some("/workspace/sk-secret-token".to_string());
    input.diagnostics.push(
        DesktopDashboardDiagnostic::new(
            DesktopDashboardDiagnosticSeverity::Warning,
            "dashboard.secret.code",
            format!("token sk-secret {}", "x".repeat(160)),
        )
        .expect("valid diagnostic"),
    );

    let snapshot = build_dashboard_snapshot(input).expect("dashboard builds");
    let serialized = serde_json::to_string(&snapshot).expect("serialize dashboard");

    assert!(snapshot.diagnostics[0].truncated);
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("dashboard.secret.code"));
    assert!(!serialized.contains(&"x".repeat(160)));
}

#[test]
fn dashboard_rejects_invalid_diagnostics() {
    let err = DesktopDashboardDiagnostic::new(
        DesktopDashboardDiagnosticSeverity::Error,
        " ",
        "settings failed",
    )
    .expect_err("empty code rejected");

    assert!(matches!(err, DesktopError::InvalidSnapshot(_)));
}

#[test]
fn dashboard_snapshot_excludes_raw_runtime_sensitive_fields() {
    let snapshot = build_dashboard_snapshot(connected_input(Vec::new())).expect("dashboard");
    let serialized = serde_json::to_string(&snapshot).expect("serialize dashboard");

    assert!(!serialized.contains("prompt"));
    assert!(!serialized.contains("tool_input"));
    assert!(!serialized.contains("command output"));
    assert!(!serialized.contains("api_key"));
}
