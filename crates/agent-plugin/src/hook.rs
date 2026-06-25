use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginHookEvent {
    TurnStarted,
    ToolCompleted,
    TurnCompleted,
    TaskFailed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginHookOutcome {
    pub hook_name: String,
    pub emitted_messages: Vec<String>,
}

#[derive(Default)]
pub struct PluginHookRunner {
    hooks: Vec<PluginHookRegistration>,
    seen: BTreeSet<(String, String)>,
}

impl PluginHookRunner {
    pub fn register<F>(&mut self, hook_name: impl Into<String>, handler: F)
    where
        F: Fn(PluginHookEvent) -> Result<PluginHookOutcome, String> + Send + Sync + 'static,
    {
        self.hooks.push(PluginHookRegistration {
            name: hook_name.into(),
            handler: Box::new(handler),
        });
    }

    pub fn run(
        &mut self,
        event: PluginHookEvent,
        correlation_id: impl Into<String>,
    ) -> Vec<PluginHookOutcome> {
        let correlation_id = correlation_id.into();
        let mut outcomes = Vec::new();

        for hook in &self.hooks {
            if !self
                .seen
                .insert((correlation_id.clone(), hook.name.clone()))
            {
                continue;
            }

            match (hook.handler)(event) {
                Ok(outcome) => outcomes.push(outcome),
                Err(message) => outcomes.push(PluginHookOutcome {
                    hook_name: hook.name.clone(),
                    emitted_messages: vec![message],
                }),
            }
        }

        outcomes
    }
}

struct PluginHookRegistration {
    name: String,
    handler: Box<dyn Fn(PluginHookEvent) -> Result<PluginHookOutcome, String> + Send + Sync>,
}
