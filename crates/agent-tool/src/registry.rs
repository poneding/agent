use std::collections::BTreeMap;
use std::sync::Arc;

use crate::definition::{ToolDefinition, ToolName};
use crate::error::ToolRegistryError;
use crate::tool::Tool;

#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<ToolName, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register<T>(&mut self, tool: T) -> Result<(), ToolRegistryError>
    where
        T: Tool + 'static,
    {
        let name = tool.definition().name;
        if self.tools.contains_key(&name) {
            return Err(ToolRegistryError::DuplicateTool(name));
        }

        self.tools.insert(name, Arc::new(tool));
        Ok(())
    }

    #[must_use]
    pub fn get(&self, name: &ToolName) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    #[must_use]
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|tool| tool.definition()).collect()
    }
}
