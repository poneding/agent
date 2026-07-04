use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCallProposed {
    pub call_id: String,
    pub tool_name: String,
    pub input_json: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PermissionDecisionRecorded {
    pub request_id: String,
    pub decision: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolExecutionStarted {
    pub call_id: String,
    pub tool_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolExecutionCompleted {
    pub call_id: String,
    pub output_json: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolExecutionFailed {
    pub call_id: String,
    pub error_code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolObservationRecorded {
    pub call_id: String,
    pub observation: String,
}
