use agent_tui::{
    ComposerAction, ComposerState, StatusBarState, TuiApp, TuiBridgeContext, TuiEffect,
    render_app_to_string,
};
use agent_types::{ClientKind, ConversationId, WorkspaceId};
use agent_ui_bridge::UiCommand;

#[test]
fn composer_reducer_accepts_text_and_submits_prompt() {
    let mut composer = ComposerState::default();
    composer.reduce(ComposerAction::InsertText(
        "summarize this crate".to_string(),
    ));

    let effect = composer.reduce(ComposerAction::Submit);

    assert_eq!(
        effect,
        TuiEffect::SubmitPrompt("summarize this crate".to_string())
    );
    assert_eq!(composer.text, "");
    assert_eq!(composer.history, vec!["summarize this crate".to_string()]);
}

#[test]
fn composer_empty_submit_returns_local_diagnostic() {
    let mut composer = ComposerState::default();

    let effect = composer.reduce(ComposerAction::Submit);

    assert_eq!(
        effect,
        TuiEffect::LocalDiagnostic("prompt cannot be empty".to_string())
    );
    assert_eq!(composer.text, "");
    assert!(composer.history.is_empty());
}

#[test]
fn composer_supports_multiline_newlines_without_submitting() {
    let mut composer = ComposerState::default();
    composer.reduce(ComposerAction::InsertText("first line".to_string()));
    composer.reduce(ComposerAction::ToggleMultiline);

    let effect = composer.reduce(ComposerAction::InsertNewline);
    composer.reduce(ComposerAction::InsertText("second line".to_string()));

    assert_eq!(effect, TuiEffect::None);
    assert!(composer.multiline);
    assert_eq!(composer.text, "first line\nsecond line");
}

#[test]
fn composer_history_navigation_is_stable_for_empty_and_multiple_entries() {
    let mut composer = ComposerState::default();

    assert_eq!(
        composer.reduce(ComposerAction::HistoryPrevious),
        TuiEffect::None
    );
    assert_eq!(composer.text, "");

    composer.reduce(ComposerAction::InsertText("first".to_string()));
    composer.reduce(ComposerAction::Submit);
    composer.reduce(ComposerAction::InsertText("second".to_string()));
    composer.reduce(ComposerAction::Submit);

    composer.reduce(ComposerAction::HistoryPrevious);
    assert_eq!(composer.text, "second");
    composer.reduce(ComposerAction::HistoryPrevious);
    assert_eq!(composer.text, "first");
    composer.reduce(ComposerAction::HistoryNext);
    assert_eq!(composer.text, "second");
    composer.reduce(ComposerAction::HistoryNext);
    assert_eq!(composer.text, "");
}

#[test]
fn composer_submission_effect_builds_start_turn_bridge_command() {
    let effect = TuiEffect::SubmitPrompt("inspect workspace".to_string());
    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();
    let context = TuiBridgeContext::new(
        workspace_id,
        conversation_id,
        ClientKind::Tui,
        "agent-tui-test",
    )
    .expect("valid context");

    let envelope = effect
        .to_ui_command(&context)
        .expect("submit effect creates command")
        .expect("submit effect maps to bridge command");

    assert_eq!(envelope.client.kind, ClientKind::Tui);
    assert_eq!(envelope.client.version, "agent-tui-test");
    assert!(matches!(
        envelope.command,
        UiCommand::StartTurn {
            workspace_id: actual_workspace_id,
            conversation_id: actual_conversation_id,
            prompt
        } if actual_workspace_id == workspace_id
            && actual_conversation_id == conversation_id
            && prompt == "inspect workspace"
    ));
}

#[test]
fn app_reducer_routes_composer_actions_and_renders_current_text() {
    let mut app = TuiApp::new(StatusBarState::default());

    let effect = app.reduce(ComposerAction::InsertText("hello tui".to_string()).into());

    assert_eq!(effect, TuiEffect::None);
    assert_eq!(app.composer.text, "hello tui");

    let rendered = render_app_to_string(&app, 80, 18).expect("render app");

    assert!(rendered.contains("hello tui"));
}
