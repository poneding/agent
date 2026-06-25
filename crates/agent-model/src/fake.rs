use async_trait::async_trait;

use crate::provider::{ModelError, ModelProvider};
use crate::request::ModelRequest;
use crate::stream::{ModelStreamEvent, ModelStreamResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeToolCall {
    pub call_id: String,
    pub tool_name: String,
    pub input_json: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeModelScript {
    pub request_id: String,
    pub deltas: Vec<String>,
    pub tool_calls: Vec<FakeToolCall>,
    pub finish_reason: String,
}

impl FakeModelScript {
    #[must_use]
    pub fn text(text: &str) -> Self {
        Self {
            request_id: "fake-request-1".to_string(),
            deltas: vec![text.to_string()],
            tool_calls: Vec::new(),
            finish_reason: "stop".to_string(),
        }
    }

    #[must_use]
    pub fn tool_call(call_id: &str, tool_name: &str, input_json: serde_json::Value) -> Self {
        Self {
            request_id: "fake-request-1".to_string(),
            deltas: Vec::new(),
            tool_calls: vec![FakeToolCall {
                call_id: call_id.to_string(),
                tool_name: tool_name.to_string(),
                input_json,
            }],
            finish_reason: "stop".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FakeModelProvider {
    script: FakeModelScript,
}

impl FakeModelProvider {
    #[must_use]
    pub fn new(script: FakeModelScript) -> Self {
        Self { script }
    }
}

#[async_trait]
impl ModelProvider for FakeModelProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelStreamResult, ModelError> {
        let mut events =
            Vec::with_capacity(self.script.deltas.len() + self.script.tool_calls.len() + 2);
        events.push(ModelStreamEvent::Started {
            request_id: self.script.request_id.clone(),
        });
        events.extend(
            self.script
                .deltas
                .iter()
                .cloned()
                .map(|text| ModelStreamEvent::Delta { text }),
        );
        events.extend(self.script.tool_calls.iter().cloned().map(|tool_call| {
            ModelStreamEvent::ToolCall {
                call_id: tool_call.call_id,
                tool_name: tool_call.tool_name,
                input_json: tool_call.input_json,
            }
        }));
        events.push(ModelStreamEvent::Completed {
            finish_reason: self.script.finish_reason.clone(),
            output_token_count: self
                .script
                .deltas
                .iter()
                .map(|delta| delta.len() as u32)
                .sum(),
        });
        Ok(events)
    }
}
