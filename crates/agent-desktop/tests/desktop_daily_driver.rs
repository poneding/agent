use agent_core::TraceEntry;
use agent_desktop::{
    DesktopApprovalReview, DesktopConfigSnapshot, DesktopConversationItem,
    DesktopConversationPanel, DesktopDashboardInput, DesktopDashboardSnapshot,
    DesktopDiffFileReview, DesktopDiffHunkPreview, DesktopDiffLineKind, DesktopDiffLinePreview,
    DesktopDiffReview, DesktopProviderSettingsSummary, DesktopSessionReadiness,
    DesktopSessionReadinessState, DesktopSettings, DesktopTaskItem, DesktopTaskMonitor,
    DesktopTurnCommandAck, DesktopTurnSubmission, DiffReviewStatus, TraceTimeline,
    build_dashboard_snapshot, build_provider_settings_summary, build_task_monitor,
    build_trace_timeline, build_workspace_snapshot,
};
use agent_types::{
    ConversationId, CorrelationId, EventId, EventSeq, SchemaVersion, SessionId, TaskId, TaskState,
    Timestamp, WorkspaceId,
};
use agent_ui_bridge::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiConfigDiagnostic, UiConfigDiagnosticSeverity,
    UiConfigSnapshot, UiEventEnvelope, UiEventPayload, UiJsonFrame, UiModelProviderCapabilities,
    UiModelProviderSummary, UiTaskSnapshot, UiTaskSummary,
};
use serde::Serialize;

#[derive(Serialize)]
struct DesktopDailyDriverSmokeFixture {
    dashboard: DesktopDashboardSnapshot,
    conversation: DesktopConversationPanel,
    approval: DesktopApprovalReview,
    diff: DesktopDiffReview,
    provider: DesktopProviderSettingsSummary,
    tasks: DesktopTaskMonitor,
    trace: TraceTimeline,
}

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-30T12:00:{second:02}Z\"")).expect("valid timestamp")
}

fn workspace_id() -> WorkspaceId {
    id("00000000-0000-0000-0000-000000000060")
}

fn conversation_id() -> ConversationId {
    id("00000000-0000-0000-0000-000000000061")
}

fn session_id() -> SessionId {
    id("00000000-0000-0000-0000-000000000062")
}

fn task_id(value: u32) -> TaskId {
    id(&format!("00000000-0000-0000-0000-{value:012}"))
}

fn command_id() -> CorrelationId {
    id("00000000-0000-0000-0000-000000000066")
}

fn event_id(value: u32) -> EventId {
    id(&format!("00000000-0000-0000-0000-{value:012}"))
}

fn event_frame(seq: u64, payload: UiEventPayload) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: event_id(u32::try_from(seq).expect("fixture seq")),
        seq: Some(EventSeq::new(seq).expect("valid sequence")),
        occurred_at: ts(u8::try_from(seq).expect("fixture timestamp")),
        conversation_id: conversation_id(),
        turn_id: None,
        task_id: None,
        correlation_id: command_id(),
        payload,
    })
}

fn config_snapshot() -> DesktopConfigSnapshot {
    let provider = UiModelProviderSummary::new(
        "fake",
        "fake",
        "fake-scripted",
        true,
        UiModelProviderCapabilities::new(true, true, true, false, Some(8192), Some(2048), None),
    )
    .expect("provider");
    let diagnostic = UiConfigDiagnostic::new(
        UiConfigDiagnosticSeverity::Info,
        "desktop.provider.fake_ready",
        "fake provider is ready for credential-free Desktop smoke",
        Some("model.provider".to_string()),
        Some("m60-smoke".to_string()),
    )
    .expect("diagnostic");
    DesktopConfigSnapshot::new(
        UiConfigSnapshot::new("fake", "fake-scripted", vec![provider], vec![diagnostic])
            .expect("config snapshot"),
    )
    .expect("desktop config")
}

fn desktop_settings() -> DesktopSettings {
    let mut settings = DesktopSettings::default();
    settings.model.provider = "fake".to_string();
    settings.model.model = "fake-scripted".to_string();
    settings
}

fn workspace_task(value: u32, state: TaskState, summary: &str, second: u8) -> DesktopTaskItem {
    DesktopTaskItem::new(
        task_id(value),
        workspace_id(),
        Some(conversation_id()),
        state,
        summary,
        ts(second),
    )
    .expect("workspace task")
}

fn task_summary(value: u32, state: TaskState, summary: &str, second: u8) -> UiTaskSummary {
    serde_json::from_value(serde_json::json!({
        "task_id": task_id(value),
        "workspace_id": workspace_id(),
        "conversation_id": conversation_id(),
        "state": state,
        "priority": "normal",
        "schedule": {
            "kind": "one_shot",
            "label": "run once"
        },
        "prompt_summary": summary,
        "latest_run_summary": format!("latest {summary}"),
        "created_at": ts(second),
        "updated_at": ts(second),
    }))
    .expect("task summary")
}

fn build_smoke_fixture() -> DesktopDailyDriverSmokeFixture {
    let config = config_snapshot();
    let settings = desktop_settings();
    let conversation_item =
        DesktopConversationItem::new(conversation_id(), "M60 Desktop golden path", ts(1), 0)
            .expect("conversation item");
    let workspace = build_workspace_snapshot(
        workspace_id(),
        Some(conversation_id()),
        vec![conversation_item.clone()],
        vec![
            workspace_task(
                63,
                TaskState::Running,
                "Apply Desktop smoke fixture update",
                4,
            ),
            workspace_task(
                64,
                TaskState::WaitingApproval,
                "Review file-edit approval",
                5,
            ),
            workspace_task(65, TaskState::Succeeded, "Run cargo xtask desktop-test", 6),
            workspace_task(66, TaskState::Failed, "Capture recoverable log evidence", 7),
        ],
    )
    .expect("workspace snapshot");
    let dashboard = build_dashboard_snapshot(DesktopDashboardInput {
        workspace,
        workspace_root_display: Some("/workspace/agent".to_string()),
        session: DesktopSessionReadiness {
            state: DesktopSessionReadinessState::Connected,
            session_id: Some(session_id()),
            conversation_id: Some(conversation_id()),
            pending_frame_count: 0,
        },
        config: config.clone(),
        pending_approval_count: 1,
        diagnostics: Vec::new(),
    })
    .expect("dashboard");

    let mut conversation = DesktopConversationPanel::new(
        workspace_id(),
        Some(conversation_id()),
        vec![conversation_item],
    )
    .expect("conversation panel");
    let submission = DesktopTurnSubmission::submitting(
        command_id(),
        conversation_id(),
        "Update the Desktop smoke fixture and run cargo xtask desktop-test",
    )
    .expect("submission");
    conversation
        .record_submission(submission)
        .expect("record submission");
    conversation
        .apply_ack(DesktopTurnCommandAck::new(command_id(), true, 4))
        .expect("ack");
    for frame in [
        event_frame(
            1,
            UiEventPayload::UserMessage {
                message: "Update the Desktop smoke fixture and run tests".to_string(),
            },
        ),
        event_frame(
            2,
            UiEventPayload::Progress {
                message: "prepared file edit preview".to_string(),
            },
        ),
        event_frame(
            3,
            UiEventPayload::ModelDelta {
                delta: "Applied the smoke fixture update.".to_string(),
            },
        ),
        event_frame(
            4,
            UiEventPayload::FinalResponse {
                message: "Desktop smoke fixture completed with tests passing.".to_string(),
            },
        ),
        UiJsonFrame::Done { event_count: 4 },
    ] {
        conversation
            .apply_frame(&frame)
            .expect("conversation frame");
    }

    let approval = DesktopApprovalReview::from_ui_request(&UiApprovalRequest {
        request_id: "00000000-0000-0000-0000-000000000067".to_string(),
        status: "pending".to_string(),
        action: "write".to_string(),
        resource_kind: "file".to_string(),
        resource: "apps/desktop/src/__tests__/desktop-smoke.test.tsx".to_string(),
        reason: "policy requires review before applying Desktop smoke fixture changes".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "file_edit".to_string(),
            resource: "apps/desktop/src/__tests__/desktop-smoke.test.tsx".to_string(),
            summary: "+assert Desktop golden path evidence\n-token sk-secret password".to_string(),
            content_hash: Some("sha256:desktop-smoke".to_string()),
            truncated: false,
            redacted: true,
        },
        max_duration_seconds: Some(120),
        expires_at: Some(ts(50)),
    })
    .expect("approval review");

    let hunk = DesktopDiffHunkPreview::new(
        40,
        40,
        vec![
            DesktopDiffLinePreview::new(
                DesktopDiffLineKind::Context,
                Some(40),
                Some(40),
                "render(<AppShell ... />)",
            )
            .expect("context line"),
            DesktopDiffLinePreview::new(
                DesktopDiffLineKind::Addition,
                None,
                Some(41),
                "+expect(screen.getByText('Desktop smoke fixture completed')).toBeInTheDocument()",
            )
            .expect("addition line"),
        ],
    )
    .expect("diff hunk");
    let diff_file = DesktopDiffFileReview::new(
        "apps/desktop/src/__tests__/desktop-smoke.test.tsx",
        12,
        2,
        "Expand Desktop smoke evidence assertions",
        vec![hunk],
    )
    .expect("diff file");
    let diff = DesktopDiffReview::new(
        "desktop-smoke-review",
        DiffReviewStatus::Pending,
        vec![diff_file],
        Some("apps/desktop/src/__tests__/desktop-smoke.test.tsx".to_string()),
    )
    .expect("diff review");

    let provider =
        build_provider_settings_summary(&config, &settings).expect("provider settings summary");
    let tasks = build_task_monitor(
        &UiTaskSnapshot::new(
            vec![
                task_summary(
                    63,
                    TaskState::Running,
                    "Apply Desktop smoke fixture update",
                    4,
                ),
                task_summary(
                    64,
                    TaskState::WaitingApproval,
                    "Review file-edit approval",
                    5,
                ),
                task_summary(65, TaskState::Succeeded, "Run cargo xtask desktop-test", 6),
                task_summary(66, TaskState::Failed, "Capture recoverable log evidence", 7),
            ],
            Vec::new(),
        )
        .expect("task snapshot"),
    )
    .expect("task monitor");
    let trace = build_trace_timeline(
        conversation_id(),
        vec![
            TraceEntry {
                seq: 1,
                kind: "context".to_string(),
                summary: "workspace dashboard ready".to_string(),
            },
            TraceEntry {
                seq: 2,
                kind: "edit_preview".to_string(),
                summary: "file edit preview prepared".to_string(),
            },
            TraceEntry {
                seq: 3,
                kind: "permission".to_string(),
                summary: "approval requested and lease displayed".to_string(),
            },
            TraceEntry {
                seq: 4,
                kind: "command_test".to_string(),
                summary: "cargo xtask desktop-test passed".to_string(),
            },
            TraceEntry {
                seq: 5,
                kind: "runtime_warning".to_string(),
                summary: "bounded warning token sk-secret password api_key".to_string(),
            },
            TraceEntry {
                seq: 6,
                kind: "final".to_string(),
                summary: "Desktop smoke fixture completed".to_string(),
            },
        ],
    )
    .expect("trace timeline");

    DesktopDailyDriverSmokeFixture {
        dashboard,
        conversation,
        approval,
        diff,
        provider,
        tasks,
        trace,
    }
}

#[test]
fn desktop_daily_driver_fixture_covers_m56_shaped_desktop_flow() {
    let fixture = build_smoke_fixture();

    assert_eq!(
        fixture.dashboard.session.state,
        DesktopSessionReadinessState::Connected
    );
    assert_eq!(fixture.dashboard.pending_approval_count, 1);
    assert!(fixture.dashboard.provider.enabled);
    assert_eq!(fixture.dashboard.task_counts.total, 4);
    assert_eq!(fixture.dashboard.task_counts.waiting_approval, 1);
    assert_eq!(fixture.dashboard.task_counts.failed, 1);
    assert_eq!(fixture.dashboard.task_counts.terminal, 2);

    let submission = fixture
        .conversation
        .current_submission
        .as_ref()
        .expect("current submission");
    assert_eq!(
        submission.prompt_preview,
        "Update the Desktop smoke fixture and run cargo xtask desktop-test"
    );
    assert!(submission.ack.as_ref().is_some_and(|ack| ack.accepted));
    assert_eq!(fixture.conversation.transcript.len(), 4);
    assert!(
        fixture
            .conversation
            .transcript
            .iter()
            .any(|entry| entry.content.contains("tests passing"))
    );

    assert!(fixture.approval.decision_enabled);
    assert_eq!(fixture.approval.lease_duration_seconds, Some(120));
    assert!(fixture.approval.preview.redacted);
    assert!(fixture.approval.preview.summary.contains("[redacted]"));

    assert!(fixture.diff.decision_enabled);
    assert_eq!(
        fixture.diff.files[0].path,
        "apps/desktop/src/__tests__/desktop-smoke.test.tsx"
    );
    assert_eq!(fixture.diff.files[0].added_lines, 12);
    assert!(!fixture.diff.files[0].hunks[0].truncated);

    assert_eq!(fixture.provider.selected_provider, "fake");
    assert!(fixture.provider.providers[0].supports_streaming);
    assert!(fixture.provider.providers[0].supports_tools);
    assert!(fixture.provider.validation.valid);

    assert_eq!(fixture.tasks.counts.total, 4);
    assert_eq!(fixture.tasks.counts.active, 1);
    assert_eq!(fixture.tasks.counts.waiting_approval, 1);
    assert_eq!(fixture.tasks.counts.failed, 1);
    assert_eq!(fixture.tasks.counts.terminal, 2);

    assert_eq!(fixture.trace.items.len(), 6);
    assert_eq!(fixture.trace.logs.len(), 6);
    assert!(
        fixture
            .trace
            .logs
            .iter()
            .any(|log| log.source == "runtime_warning" && log.message.contains("[redacted]"))
    );
}

#[test]
fn desktop_daily_driver_fixture_serializes_without_secret_leaks() {
    let fixture = build_smoke_fixture();
    let serialized = serde_json::to_string(&fixture).expect("serialize fixture");

    assert!(serialized.contains("Desktop smoke fixture completed"));
    assert!(serialized.contains("[redacted]"));
    for forbidden in [
        "sk-secret",
        "api_key",
        "token",
        "password",
        "secret",
        "tool_input",
        "input_json",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "serialized fixture leaked {forbidden}: {serialized}"
        );
    }
}
