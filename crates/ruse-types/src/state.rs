use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnState {
    Accepted,
    Prepared,
    ContextBuilt,
    ModelStreaming,
    ToolProposed,
    PermissionEvaluating,
    ToolExecuting,
    Observed,
    Finalizing,
    Persisted,
    Done,
    WaitingApproval,
    WaitingIo,
    Cancelled,
    Failed,
    Recovered,
}

impl TurnState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled | Self::Failed)
    }

    #[must_use]
    pub const fn is_resumable(self) -> bool {
        matches!(
            self,
            Self::WaitingApproval | Self::WaitingIo | Self::Recovered
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Created,
    Queued,
    Running,
    WaitingApproval,
    Sleeping,
    Paused,
    Succeeded,
    Failed,
    Cancelled,
    Expired,
}

impl TaskState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Expired
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_state_uses_snake_case() {
        let json = serde_json::to_string(&TurnState::ModelStreaming).expect("serialize state");
        assert_eq!(json, "\"model_streaming\"");
    }

    #[test]
    fn terminal_states_are_marked() {
        assert!(TurnState::Done.is_terminal());
        assert!(TaskState::Succeeded.is_terminal());
        assert!(!TaskState::Running.is_terminal());
    }
}
