use agent_model::{ModelProvider, ModelRequest, ModelStreamEvent};
use agent_types::{
    ActorKind, ActorRef, AgentError, ContextAssembled, FinalResponseProduced, ModelFinishReason,
    ModelOutputDelta, ModelStreamCompleted, ModelStreamStarted, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, Timestamp, ToolCallProposed, TurnCompleted, TurnFailed, TurnStarted, TurnState,
    TurnStateChanged, UserInputReceived,
};
use thiserror::Error;

use crate::context::ContextAssembler;
use crate::runtime::RuntimeIds;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnInput {
    pub ids: RuntimeIds,
    pub user_message: String,
    pub provider: String,
    pub model: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnOutcome {
    pub final_message: String,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TurnError {
    #[error("model stream failed: {message}")]
    Model {
        message: String,
        events: Vec<RuntimeEvent>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct TurnEngine {
    context: ContextAssembler,
}

impl TurnEngine {
    #[must_use]
    pub const fn new(context: ContextAssembler) -> Self {
        Self { context }
    }

    pub async fn run<P>(
        &self,
        input: TurnInput,
        model_provider: &P,
    ) -> Result<TurnOutcome, TurnError>
    where
        P: ModelProvider + ?Sized,
    {
        let mut events = vec![
            event(
                input.ids,
                user_actor(),
                RuntimeEventPayload::UserInputReceived(UserInputReceived {
                    message: input.user_message.clone(),
                }),
            ),
            event(
                input.ids,
                ActorRef::system(),
                RuntimeEventPayload::TurnStarted(TurnStarted {
                    input_summary: input.user_message.clone(),
                }),
            ),
            state_changed(
                input.ids,
                TurnState::Accepted,
                TurnState::ContextBuilt,
                "context assembled",
            ),
        ];

        let messages = self.context.assemble(&input.user_message);
        events.push(event(
            input.ids,
            ActorRef::system(),
            RuntimeEventPayload::ContextAssembled(ContextAssembled {
                message_count: messages.len() as u32,
                estimated_tokens: estimate_tokens(&messages),
            }),
        ));
        events.push(state_changed(
            input.ids,
            TurnState::ContextBuilt,
            TurnState::ModelStreaming,
            "model stream started",
        ));

        let request = ModelRequest {
            provider: input.provider.clone(),
            model: input.model.clone(),
            conversation_id: input.ids.conversation_id,
            turn_id: input.ids.turn_id,
            correlation_id: input.ids.correlation_id,
            messages,
        };

        let stream = match model_provider.stream(request).await {
            Ok(stream) => stream,
            Err(error) => {
                let message = error.to_string();
                events.push(state_changed(
                    input.ids,
                    TurnState::ModelStreaming,
                    TurnState::Failed,
                    "model stream failed",
                ));
                events.push(event(
                    input.ids,
                    ActorRef::system(),
                    RuntimeEventPayload::TurnFailed(TurnFailed {
                        state: TurnState::Failed,
                        error: AgentError::new("model_stream_failed", message.clone(), true),
                    }),
                ));
                return Err(TurnError::Model { message, events });
            }
        };

        let mut final_message = String::new();

        for stream_event in stream {
            match stream_event {
                ModelStreamEvent::Started { request_id } => events.push(event(
                    input.ids,
                    ActorRef::system(),
                    RuntimeEventPayload::ModelStreamStarted(ModelStreamStarted {
                        provider: input.provider.clone(),
                        model: input.model.clone(),
                        request_id,
                    }),
                )),
                ModelStreamEvent::Delta { text } => {
                    final_message.push_str(&text);
                    events.push(event(
                        input.ids,
                        ActorRef::system(),
                        RuntimeEventPayload::ModelOutputDelta(ModelOutputDelta { delta: text }),
                    ));
                }
                ModelStreamEvent::ToolCall {
                    call_id,
                    tool_name,
                    input_json,
                } => events.push(event(
                    input.ids,
                    ActorRef::system(),
                    RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
                        call_id,
                        tool_name,
                        input_json,
                    }),
                )),
                ModelStreamEvent::Completed {
                    finish_reason,
                    output_token_count,
                } => events.push(event(
                    input.ids,
                    ActorRef::system(),
                    RuntimeEventPayload::ModelStreamCompleted(ModelStreamCompleted {
                        finish_reason: model_finish_reason(&finish_reason),
                        output_token_count,
                    }),
                )),
            }
        }

        events.push(state_changed(
            input.ids,
            TurnState::ModelStreaming,
            TurnState::Finalizing,
            "model output collected",
        ));
        events.push(event(
            input.ids,
            ActorRef::system(),
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: final_message.clone(),
            }),
        ));
        events.push(state_changed(
            input.ids,
            TurnState::Finalizing,
            TurnState::Done,
            "final response produced",
        ));
        events.push(event(
            input.ids,
            ActorRef::system(),
            RuntimeEventPayload::TurnCompleted(TurnCompleted {
                final_state: TurnState::Done,
                summary: "turn completed".to_string(),
            }),
        ));

        Ok(TurnOutcome {
            final_message,
            events,
        })
    }
}

fn event(ids: RuntimeIds, actor: ActorRef, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        turn_id: Some(ids.turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor,
        correlation_id: ids.correlation_id,
        causation_id: None,
        payload,
    }
}

fn state_changed(ids: RuntimeIds, from: TurnState, to: TurnState, reason: &str) -> RuntimeEvent {
    event(
        ids,
        ActorRef::system(),
        RuntimeEventPayload::TurnStateChanged(TurnStateChanged {
            from,
            to,
            reason: reason.to_string(),
        }),
    )
}

fn user_actor() -> ActorRef {
    ActorRef {
        kind: ActorKind::User,
        id: "user".to_string(),
        display_name: Some("user".to_string()),
    }
}

fn estimate_tokens(messages: &[String]) -> u32 {
    messages.iter().map(|message| message.len() as u32).sum()
}

fn model_finish_reason(reason: &str) -> ModelFinishReason {
    match reason {
        "stop" => ModelFinishReason::Stop,
        "length" => ModelFinishReason::Length,
        "cancelled" => ModelFinishReason::Cancelled,
        "error" => ModelFinishReason::Error,
        _ => ModelFinishReason::Error,
    }
}
