use async_trait::async_trait;

use crate::provider::{ModelError, ModelProvider};
use crate::request::ModelRequest;
use crate::stream::{ModelStreamEvent, ModelStreamResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeModelScript {
    pub request_id: String,
    pub deltas: Vec<String>,
    pub finish_reason: String,
}

impl FakeModelScript {
    #[must_use]
    pub fn text(text: &str) -> Self {
        Self {
            request_id: "fake-request-1".to_string(),
            deltas: vec![text.to_string()],
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
        let mut events = Vec::with_capacity(self.script.deltas.len() + 2);
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
