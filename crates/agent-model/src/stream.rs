use std::pin::Pin;

use futures_core::Stream;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};

use crate::provider::ModelError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStreamEvent {
    Started {
        request_id: String,
    },
    Delta {
        text: String,
    },
    ToolCall {
        call_id: String,
        tool_name: String,
        input_json: serde_json::Value,
    },
    Completed {
        finish_reason: String,
        output_token_count: u32,
    },
}

pub type ModelEventStream =
    Pin<Box<dyn Stream<Item = Result<ModelStreamEvent, ModelError>> + Send>>;

pub type ModelStreamResult = Vec<ModelStreamEvent>;

#[must_use]
pub fn model_events_to_stream(events: ModelStreamResult) -> ModelEventStream {
    Box::pin(stream::iter(events.into_iter().map(Ok::<_, ModelError>)))
}

#[must_use]
pub fn model_error_to_stream(error: ModelError) -> ModelEventStream {
    Box::pin(stream::iter([Err(error)]))
}

pub async fn collect_model_event_stream(
    mut stream: ModelEventStream,
) -> Result<ModelStreamResult, ModelError> {
    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        events.push(item?);
    }
    Ok(events)
}
