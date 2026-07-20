use serde::{Deserialize, Serialize};

use crate::error::PolicyError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityAction {
    Read,
    Search,
    Execute,
    Network,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    File,
    Workspace,
    Command,
    Network,
    Tool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ResourceScope {
    pub kind: ResourceKind,
    pub pattern: String,
}

impl ResourceScope {
    pub fn new(kind: ResourceKind, pattern: impl Into<String>) -> Result<Self, PolicyError> {
        let pattern = pattern.into();
        if pattern.trim().is_empty() {
            return Err(PolicyError::InvalidRequest(
                "resource scope pattern cannot be empty".to_string(),
            ));
        }

        Ok(Self { kind, pattern })
    }
}
