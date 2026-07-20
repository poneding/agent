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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolExecutionStarted {
    pub call_id: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_profile: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolExecutionCompleted {
    pub call_id: String,
    pub output_json: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolExecutionFailed {
    pub call_id: String,
    pub error_code: String,
    pub message: String,
    pub recoverable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolObservationRecorded {
    pub call_id: String,
    pub observation: String,
}
