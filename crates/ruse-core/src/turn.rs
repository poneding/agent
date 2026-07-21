use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures_util::{Stream, StreamExt, stream};
use ruse_context::PromptMessage;
use ruse_model::{
    ModelMessage, ModelProvider, ModelRequest, ModelStreamEvent, ModelToolDefinition,
};
use ruse_types::{
    ActorKind, ActorRef, AgentError, ContextAssembled, FinalResponseProduced, ModelFinishReason,
    ModelOutputDelta, ModelReasoningSummaryDelta, ModelStreamCompleted, ModelStreamStarted,
    ReasoningEffort, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, ToolCallProposed,
    TurnCompleted, TurnFailed, TurnStarted, TurnState, TurnStateChanged, UserInputReceived,
};
use thiserror::Error;

use crate::context::ContextAssembler;
use crate::runtime::RuntimeIds;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnInput {
    pub ids: RuntimeIds,
    pub user_message: String,
    pub conversation_history: Vec<PromptMessage>,
    pub continuation_messages: Vec<ModelMessage>,
    pub provider: String,
    pub model: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnOutcome {
    pub final_message: String,
    pub events: Vec<RuntimeEvent>,
}

pub type TurnEventStream = Pin<Box<dyn Stream<Item = Result<RuntimeEvent, TurnError>> + Send>>;

#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TurnStreamOptions {
    pub cancellation: CancellationToken,
    pub tools: Vec<ModelToolDefinition>,
    pub reasoning_effort: Option<ReasoningEffort>,
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

    /// Coding-agent defaults: AGENTS.md/CLAUDE.md + git status when present.
    #[must_use]
    pub fn for_workspace(workspace_root: &std::path::Path) -> Self {
        Self {
            context: crate::context::coding_context_assembler(workspace_root),
        }
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

        let mut messages = self
            .context
            .assemble_messages_with_history(&input.user_message, &input.conversation_history)
            .into_iter()
            .map(model_message_from_prompt)
            .collect::<Vec<_>>();
        messages.extend(input.continuation_messages.clone());
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
            reasoning_effort: None,
            tools: Vec::new(),
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

        let mut stream = stream;
        while let Some(stream_event) = stream.next().await {
            let stream_event = match stream_event {
                Ok(stream_event) => stream_event,
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
                ModelStreamEvent::ReasoningSummaryDelta { text } => events.push(event(
                    input.ids,
                    ActorRef::system(),
                    RuntimeEventPayload::ModelReasoningSummaryDelta(ModelReasoningSummaryDelta {
                        delta: text,
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

    #[must_use]
    pub fn run_stream(
        &self,
        input: TurnInput,
        model_provider: Arc<dyn ModelProvider>,
    ) -> TurnEventStream {
        self.run_stream_with_options(input, model_provider, TurnStreamOptions::default())
    }

    #[must_use]
    pub fn run_stream_with_options(
        &self,
        input: TurnInput,
        model_provider: Arc<dyn ModelProvider>,
        options: TurnStreamOptions,
    ) -> TurnEventStream {
        let mut pending = VecDeque::from([
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
        ]);

        let mut messages = self
            .context
            .assemble_messages_with_history(&input.user_message, &input.conversation_history)
            .into_iter()
            .map(model_message_from_prompt)
            .collect::<Vec<_>>();
        messages.extend(input.continuation_messages);
        pending.push_back(event(
            input.ids,
            ActorRef::system(),
            RuntimeEventPayload::ContextAssembled(ContextAssembled {
                message_count: messages.len() as u32,
                estimated_tokens: estimate_tokens(&messages),
            }),
        ));
        pending.push_back(state_changed(
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
            reasoning_effort: options.reasoning_effort,
            tools: options.tools,
        };
        let state = TurnStreamState {
            ids: input.ids,
            provider_name: input.provider,
            model: input.model,
            model_provider,
            request,
            model_stream: None,
            final_message: String::new(),
            pending,
            terminal_error: None,
            cancellation: options.cancellation,
            phase: TurnStreamPhase::OpenModel,
        };

        Box::pin(stream::unfold(state, next_turn_stream_item))
    }
}

struct TurnStreamState {
    ids: RuntimeIds,
    provider_name: String,
    model: String,
    model_provider: Arc<dyn ModelProvider>,
    request: ModelRequest,
    model_stream: Option<ruse_model::ModelEventStream>,
    final_message: String,
    pending: VecDeque<RuntimeEvent>,
    terminal_error: Option<String>,
    cancellation: CancellationToken,
    phase: TurnStreamPhase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TurnStreamPhase {
    OpenModel,
    Model,
    Done,
}

async fn next_turn_stream_item(
    mut state: TurnStreamState,
) -> Option<(Result<RuntimeEvent, TurnError>, TurnStreamState)> {
    loop {
        if let Some(event) = state.pending.pop_front() {
            return Some((Ok(event), state));
        }

        if let Some(message) = state.terminal_error.take() {
            state.phase = TurnStreamPhase::Done;
            return Some((
                Err(TurnError::Model {
                    message,
                    events: Vec::new(),
                }),
                state,
            ));
        }

        if state.cancellation.is_cancelled() && state.phase != TurnStreamPhase::Done {
            queue_cancelled_events(&mut state);
            continue;
        }

        match state.phase {
            TurnStreamPhase::OpenModel => {
                match state.model_provider.stream(state.request.clone()).await {
                    Ok(model_stream) => {
                        state.model_stream = Some(model_stream);
                        state.phase = TurnStreamPhase::Model;
                    }
                    Err(error) => queue_model_failure(&mut state, error.to_string()),
                }
            }
            TurnStreamPhase::Model => {
                let Some(model_stream) = state.model_stream.as_mut() else {
                    queue_model_failure(&mut state, "model stream was not initialized".to_string());
                    continue;
                };
                match model_stream.next().await {
                    Some(Ok(stream_event)) => handle_model_stream_event(&mut state, stream_event),
                    Some(Err(error)) => queue_model_failure(&mut state, error.to_string()),
                    None => queue_final_events(&mut state),
                }
            }
            TurnStreamPhase::Done => return None,
        }
    }
}

fn queue_cancelled_events(state: &mut TurnStreamState) {
    state.pending.push_back(state_changed(
        state.ids,
        TurnState::ModelStreaming,
        TurnState::Cancelled,
        "model stream cancelled",
    ));
    state.pending.push_back(event(
        state.ids,
        ActorRef::system(),
        RuntimeEventPayload::ModelStreamCompleted(ModelStreamCompleted {
            finish_reason: ModelFinishReason::Cancelled,
            output_token_count: 0,
        }),
    ));
    state.pending.push_back(event(
        state.ids,
        ActorRef::system(),
        RuntimeEventPayload::TurnCompleted(TurnCompleted {
            final_state: TurnState::Cancelled,
            summary: "turn cancelled".to_string(),
        }),
    ));
    state.phase = TurnStreamPhase::Done;
}

fn handle_model_stream_event(state: &mut TurnStreamState, stream_event: ModelStreamEvent) {
    match stream_event {
        ModelStreamEvent::Started { request_id } => state.pending.push_back(event(
            state.ids,
            ActorRef::system(),
            RuntimeEventPayload::ModelStreamStarted(ModelStreamStarted {
                provider: state.provider_name.clone(),
                model: state.model.clone(),
                request_id,
            }),
        )),
        ModelStreamEvent::ReasoningSummaryDelta { text } => state.pending.push_back(event(
            state.ids,
            ActorRef::system(),
            RuntimeEventPayload::ModelReasoningSummaryDelta(ModelReasoningSummaryDelta {
                delta: text,
            }),
        )),
        ModelStreamEvent::Delta { text } => {
            state.final_message.push_str(&text);
            state.pending.push_back(event(
                state.ids,
                ActorRef::system(),
                RuntimeEventPayload::ModelOutputDelta(ModelOutputDelta { delta: text }),
            ));
        }
        ModelStreamEvent::ToolCall {
            call_id,
            tool_name,
            input_json,
        } => state.pending.push_back(event(
            state.ids,
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
        } => state.pending.push_back(event(
            state.ids,
            ActorRef::system(),
            RuntimeEventPayload::ModelStreamCompleted(ModelStreamCompleted {
                finish_reason: model_finish_reason(&finish_reason),
                output_token_count,
            }),
        )),
    }
}

fn queue_final_events(state: &mut TurnStreamState) {
    state.pending.push_back(state_changed(
        state.ids,
        TurnState::ModelStreaming,
        TurnState::Finalizing,
        "model output collected",
    ));
    state.pending.push_back(event(
        state.ids,
        ActorRef::system(),
        RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
            message: state.final_message.clone(),
        }),
    ));
    state.pending.push_back(state_changed(
        state.ids,
        TurnState::Finalizing,
        TurnState::Done,
        "final response produced",
    ));
    state.pending.push_back(event(
        state.ids,
        ActorRef::system(),
        RuntimeEventPayload::TurnCompleted(TurnCompleted {
            final_state: TurnState::Done,
            summary: "turn completed".to_string(),
        }),
    ));
    state.phase = TurnStreamPhase::Done;
}

fn queue_model_failure(state: &mut TurnStreamState, message: String) {
    state.pending.push_back(state_changed(
        state.ids,
        TurnState::ModelStreaming,
        TurnState::Failed,
        "model stream failed",
    ));
    state.pending.push_back(event(
        state.ids,
        ActorRef::system(),
        RuntimeEventPayload::TurnFailed(TurnFailed {
            state: TurnState::Failed,
            error: AgentError::new("model_stream_failed", message.clone(), true),
        }),
    ));
    state.terminal_error = Some(message);
    state.phase = TurnStreamPhase::Done;
}

fn event(ids: RuntimeIds, actor: ActorRef, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
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

fn model_message_from_prompt(message: PromptMessage) -> ModelMessage {
    match message.role.as_str() {
        "user" => ModelMessage::user(message.content),
        "assistant" => ModelMessage::assistant(message.content),
        _ => ModelMessage::system(message.content),
    }
}

fn estimate_tokens(messages: &[ModelMessage]) -> u32 {
    messages
        .iter()
        .filter_map(ModelMessage::text_content)
        .map(|message| message.len() as u32)
        .sum()
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
