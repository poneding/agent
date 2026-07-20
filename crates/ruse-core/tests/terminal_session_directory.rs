use ruse_core::{
    RepoGitState, TerminalSessionCatalog, TerminalSessionRecord, TerminalSessionStatus,
    build_terminal_session_directory, summarize_terminal_session_title,
    validate_terminal_session_restore,
};
use ruse_storage::{EventStore, InMemoryEventStore};
use ruse_types::{
    ActorRef, ClientKind, ConversationId, CorrelationId, EventId, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, UserInputReceived, WorkspaceId,
};

fn ts(day: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-07-{day:02}T00:00:00Z\"")).expect("timestamp")
}

fn record(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    title: &str,
    last_event_at: Timestamp,
) -> TerminalSessionRecord {
    TerminalSessionRecord {
        workspace_id,
        conversation_id,
        workspace_root_display: format!("workspace-{conversation_id}"),
        title: title.to_string(),
        client_kind: ClientKind::Cli,
        client_version: "test".to_string(),
        provider_id: "fake".to_string(),
        model_id: "fake-scripted".to_string(),
        git_state: RepoGitState::Clean,
        last_event_at,
        pending_approval_count: 0,
        running_command_count: 0,
        task_count: 0,
        failed: false,
    }
}

fn user_event(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    message: &str,
    occurred_at: Timestamp,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
        seq: None,
        occurred_at,
        workspace_id,
        conversation_id,
        turn_id: None,
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload: RuntimeEventPayload::UserInputReceived(UserInputReceived {
            message: message.to_string(),
        }),
    }
}

#[test]
fn terminal_session_directory_empty_report_is_successful() {
    let report = build_terminal_session_directory(Vec::new()).expect("empty report");

    assert!(report.sessions.is_empty());
    assert!(report.diagnostics.is_empty());
}

#[test]
fn terminal_session_directory_sorts_and_collapses_duplicate_sessions() {
    let workspace_one = WorkspaceId::new();
    let conversation_one = ConversationId::new();
    let workspace_two = WorkspaceId::new();
    let conversation_two = ConversationId::new();
    let duplicate_old = record(workspace_one, conversation_one, "old", ts(1));
    let mut duplicate_new = record(workspace_one, conversation_one, "new", ts(3));
    duplicate_new.running_command_count = 1;
    let later = record(workspace_two, conversation_two, "later", ts(2));

    let report = build_terminal_session_directory(vec![duplicate_old, later, duplicate_new])
        .expect("directory");

    assert_eq!(report.sessions.len(), 2);
    assert_eq!(report.sessions[0].conversation_id, conversation_one);
    assert_eq!(report.sessions[0].title, "new");
    assert_eq!(
        report.sessions[0].status,
        TerminalSessionStatus::RunningCommand
    );
    assert_eq!(report.sessions[1].conversation_id, conversation_two);
}

#[test]
fn terminal_session_directory_classifies_status_from_runtime_counts() {
    let mut waiting = record(WorkspaceId::new(), ConversationId::new(), "waiting", ts(1));
    waiting.pending_approval_count = 1;
    let mut running = record(WorkspaceId::new(), ConversationId::new(), "running", ts(2));
    running.running_command_count = 1;
    let mut tasked = record(WorkspaceId::new(), ConversationId::new(), "tasks", ts(3));
    tasked.task_count = 2;
    let mut failed = record(WorkspaceId::new(), ConversationId::new(), "failed", ts(4));
    failed.failed = true;

    let report = build_terminal_session_directory(vec![waiting, running, tasked, failed])
        .expect("directory");

    let statuses = report
        .sessions
        .iter()
        .map(|session| (session.title.as_str(), session.status))
        .collect::<Vec<_>>();

    assert!(statuses.contains(&("waiting", TerminalSessionStatus::WaitingApproval)));
    assert!(statuses.contains(&("running", TerminalSessionStatus::RunningCommand)));
    assert!(statuses.contains(&("tasks", TerminalSessionStatus::HasTasks)));
    assert!(statuses.contains(&("failed", TerminalSessionStatus::Failed)));
}

#[test]
fn terminal_session_restore_validates_target_conversation() {
    let conversation = ConversationId::new();
    let report = build_terminal_session_directory(vec![record(
        WorkspaceId::new(),
        conversation,
        "one",
        ts(1),
    )])
    .expect("directory");

    let restored =
        validate_terminal_session_restore(&report, conversation).expect("restore target");

    assert_eq!(restored.conversation_id, conversation);
    assert!(validate_terminal_session_restore(&report, ConversationId::new()).is_err());
}

#[test]
fn terminal_session_catalog_titles_first_user_message_and_preserves_it() {
    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();
    let mut catalog = TerminalSessionCatalog::new(vec![record(
        workspace_id,
        conversation_id,
        "Repo session",
        ts(1),
    )]);

    catalog.apply_event(&user_event(
        workspace_id,
        conversation_id,
        "修复会话切换时没有加载消息历史。后面是无关细节 sk-title-secret",
        ts(2),
    ));
    catalog.apply_event(&user_event(
        workspace_id,
        conversation_id,
        "this later message must not replace the title",
        ts(3),
    ));

    let report = catalog.report().expect("catalog report");
    assert_eq!(report.sessions.len(), 1);
    assert_eq!(report.sessions[0].title, "修复会话切换时没有加载消息历史");
    assert!(!report.sessions[0].title.contains("sk-title-secret"));
    assert_eq!(report.sessions[0].last_event_at, ts(3));
}

#[test]
fn terminal_session_catalog_rebuilds_title_from_persisted_events() {
    let workspace_id = WorkspaceId::new();
    let active_conversation_id = ConversationId::new();
    let persisted_conversation_id = ConversationId::new();
    let mut store = InMemoryEventStore::default();
    let mut started = user_event(
        workspace_id,
        persisted_conversation_id,
        "placeholder",
        ts(1),
    );
    started.payload = RuntimeEventPayload::ConversationStarted(ruse_types::ConversationStarted {
        title: "Repo session".to_string(),
    });
    store.append(started).expect("append conversation start");
    store
        .append(user_event(
            workspace_id,
            persisted_conversation_id,
            "Investigate persisted session history. Include later details",
            ts(2),
        ))
        .expect("append persisted user message");

    let mut catalog = TerminalSessionCatalog::new(vec![record(
        workspace_id,
        active_conversation_id,
        "New session",
        ts(3),
    )]);
    catalog
        .hydrate_from_store(&store)
        .expect("hydrate persisted sessions");

    let report = catalog.report().expect("catalog report");
    let restored = report
        .sessions
        .iter()
        .find(|session| session.conversation_id == persisted_conversation_id)
        .expect("persisted session");
    assert_eq!(restored.title, "Investigate persisted session history");
}

#[test]
fn terminal_session_catalog_new_conversation_stays_in_workspace() {
    let workspace_id = WorkspaceId::new();
    let current = ConversationId::new();
    let next = ConversationId::new();
    let mut catalog =
        TerminalSessionCatalog::new(vec![record(workspace_id, current, "Current", ts(1))]);

    let created = catalog
        .start_new_conversation(current, next, ts(2))
        .expect("new conversation");
    let report = catalog.report().expect("catalog report");

    assert_eq!(created.workspace_id, workspace_id);
    assert_eq!(created.conversation_id, next);
    assert_eq!(created.title, "New session");
    assert_eq!(report.sessions.len(), 2);
}

#[test]
fn terminal_session_title_is_bounded_and_redacted() {
    let title = summarize_terminal_session_title(
        "sk-title-secret please investigate this very long conversation title that should be bounded before it reaches a terminal session picker",
    );

    assert!(!title.contains("sk-title-secret"));
    assert!(title.chars().count() <= 59);
}

#[test]
fn terminal_session_directory_redacts_and_bounds_public_text() {
    let mut unsafe_record = record(
        WorkspaceId::new(),
        ConversationId::new(),
        "sk-test-secret title",
        ts(1),
    );
    unsafe_record.workspace_root_display = "/tmp/api_key/project".to_string();
    unsafe_record.provider_id = "provider-secret".to_string();
    unsafe_record.model_id = "model-token".to_string();
    unsafe_record.client_version = "client-secret-version".to_string();

    let report = build_terminal_session_directory(vec![unsafe_record]).expect("directory");
    let rendered = serde_json::to_string(&report).expect("serialize report");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("provider-secret"));
    assert!(!rendered.contains("model-token"));
    assert!(!rendered.contains("client-secret-version"));
    assert!(report.sessions[0].workspace_root_display.chars().count() <= 160);
}
