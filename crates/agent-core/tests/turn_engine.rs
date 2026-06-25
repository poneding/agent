use agent_core::{RuntimeIds, TurnEngine, TurnError, TurnInput};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelError, ModelProvider, ModelRequest, ModelStreamResult,
};
use agent_types::{
    ConversationId, CorrelationId, RuntimeEvent, RuntimeEventPayload, TurnId, TurnState,
    WorkspaceId,
};
use async_trait::async_trait;

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn turn_input(ids: RuntimeIds) -> TurnInput {
    TurnInput {
        ids,
        user_message: "build a tiny runtime".to_string(),
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
    }
}

fn fake_provider() -> FakeModelProvider {
    FakeModelProvider::new(FakeModelScript {
        request_id: "turn-request-1".to_string(),
        deltas: vec!["hel".to_string(), "lo".to_string()],
        tool_calls: Vec::new(),
        finish_reason: "stop".to_string(),
    })
}

fn state_changes(events: &[RuntimeEvent]) -> Vec<(TurnState, TurnState)> {
    events
        .iter()
        .filter_map(|event| match &event.payload {
            RuntimeEventPayload::TurnStateChanged(change) => Some((change.from, change.to)),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn turn_engine_starts_from_accepted_input() {
    let ids = runtime_ids();
    let outcome = TurnEngine::default()
        .run(turn_input(ids), &fake_provider())
        .await
        .expect("run turn");

    assert!(matches!(
        &outcome.events[0].payload,
        RuntimeEventPayload::UserInputReceived(input) if input.message == "build a tiny runtime"
    ));
    assert!(matches!(
        &outcome.events[1].payload,
        RuntimeEventPayload::TurnStarted(started)
            if started.input_summary == "build a tiny runtime"
    ));
    assert_eq!(
        state_changes(&outcome.events).first().copied(),
        Some((TurnState::Accepted, TurnState::ContextBuilt))
    );
}

#[tokio::test]
async fn turn_engine_emits_ordered_state_changes_and_reaches_done() {
    let outcome = TurnEngine::default()
        .run(turn_input(runtime_ids()), &fake_provider())
        .await
        .expect("run turn");

    assert_eq!(
        state_changes(&outcome.events),
        vec![
            (TurnState::Accepted, TurnState::ContextBuilt),
            (TurnState::ContextBuilt, TurnState::ModelStreaming),
            (TurnState::ModelStreaming, TurnState::Finalizing),
            (TurnState::Finalizing, TurnState::Done),
        ]
    );
    assert!(matches!(
        outcome.events.last().map(|event| &event.payload),
        Some(RuntimeEventPayload::TurnCompleted(completed))
            if completed.final_state == TurnState::Done
    ));
}

#[tokio::test]
async fn turn_engine_aggregates_fake_model_output() {
    let outcome = TurnEngine::default()
        .run(turn_input(runtime_ids()), &fake_provider())
        .await
        .expect("run turn");

    let deltas: Vec<&str> = outcome
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            RuntimeEventPayload::ModelOutputDelta(delta) => Some(delta.delta.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(deltas, vec!["hel", "lo"]);
    assert_eq!(outcome.final_message, "hello");
    assert!(outcome.events.iter().any(|event| matches!(
        &event.payload,
        RuntimeEventPayload::FinalResponseProduced(final_response)
            if final_response.message == "hello"
    )));
}

struct FailingProvider;

#[async_trait]
impl ModelProvider for FailingProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelStreamResult, ModelError> {
        Err(ModelError::Script("provider unavailable".to_string()))
    }
}

#[tokio::test]
async fn turn_engine_model_error_returns_model_error_and_failed_event() {
    let error = TurnEngine::default()
        .run(turn_input(runtime_ids()), &FailingProvider)
        .await
        .expect_err("model error");

    match error {
        TurnError::Model { message, events } => {
            assert_eq!(message, "model script failed: provider unavailable");
            assert!(events.iter().any(|event| matches!(
                &event.payload,
                RuntimeEventPayload::TurnFailed(failed)
                    if failed.state == TurnState::Failed
                        && failed.error.code == "model_stream_failed"
            )));
        }
    }
}
