use std::str::FromStr;

use ruse_types::ApprovalRequestId;
use ruse_ui_bridge::{
    UiApprovalDecision, UiApprovalRequest, UiBridgeError, UiCommand, UiJsonFrame, encode_json_frame,
};
use thiserror::Error;

use crate::CliOutput;

const CLI_ALLOW_REASON: &str = "approved from CLI";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliApprovalChoice {
    Allow { lease_duration_seconds: Option<u64> },
    Deny { reason: String },
    Abort,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CliApprovalParseError {
    #[error("approval choice cannot be empty")]
    Empty,
    #[error("unknown approval choice: {0}")]
    UnknownChoice(String),
    #[error("invalid approval duration: {0}")]
    InvalidDuration(String),
    #[error("deny requires a reason")]
    MissingDenyReason,
}

#[derive(Debug, Error)]
pub enum CliApprovalError {
    #[error(transparent)]
    Parse(#[from] CliApprovalParseError),
    #[error("invalid approval request id: {0}")]
    InvalidRequestId(String),
}

pub fn parse_approval_input(input: &str) -> Result<CliApprovalChoice, CliApprovalParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(CliApprovalParseError::Empty);
    }

    let mut parts = input.split_whitespace();
    let choice = parts.next().expect("non-empty input has first token");
    let choice_lower = choice.to_ascii_lowercase();

    match choice_lower.as_str() {
        "allow" | "a" => parse_allow_duration(parts.collect()),
        "deny" | "d" => {
            let reason = parts.collect::<Vec<_>>().join(" ");
            if reason.trim().is_empty() {
                Err(CliApprovalParseError::MissingDenyReason)
            } else {
                Ok(CliApprovalChoice::Deny { reason })
            }
        }
        "abort" | "cancel" | "q" | "quit" => Ok(CliApprovalChoice::Abort),
        _ => Err(CliApprovalParseError::UnknownChoice(choice.to_string())),
    }
}

pub fn approval_command_from_input(
    request: &UiApprovalRequest,
    input: &str,
) -> Result<Option<UiCommand>, CliApprovalError> {
    let request_id = ApprovalRequestId::from_str(&request.request_id)
        .map_err(|_| CliApprovalError::InvalidRequestId(request.request_id.clone()))?;

    match parse_approval_input(input)? {
        CliApprovalChoice::Allow {
            lease_duration_seconds,
        } => Ok(Some(UiCommand::ResolveApproval {
            request_id,
            decision: UiApprovalDecision::Allow,
            reason: CLI_ALLOW_REASON.to_string(),
            lease_duration_seconds,
        })),
        CliApprovalChoice::Deny { reason } => Ok(Some(UiCommand::ResolveApproval {
            request_id,
            decision: UiApprovalDecision::Deny,
            reason,
            lease_duration_seconds: None,
        })),
        CliApprovalChoice::Abort => Ok(None),
    }
}

#[must_use]
pub fn render_approval_prompt(request: &UiApprovalRequest) -> String {
    let mut output = String::new();
    output.push_str("Approval required\n");
    output.push_str(&format!("Request: {}\n", request.request_id));
    output.push_str(&format!("Status: {}\n", request.status));
    output.push_str(&format!("Action: {}\n", request.action));
    output.push_str(&format!(
        "Resource: {} {}\n",
        request.resource_kind, request.resource
    ));
    output.push_str(&format!("Reason: {}\n", request.reason));
    output.push_str(&format!("Preview kind: {}\n", request.preview.kind));
    output.push_str(&format!("Preview: {}\n", request.preview.summary));
    output.push_str(&format!("Preview resource: {}\n", request.preview.resource));

    if let Some(content_hash) = &request.preview.content_hash {
        output.push_str(&format!("Hash: {content_hash}\n"));
    }

    output.push_str(&format!(
        "Truncated: {}\n",
        yes_no(request.preview.truncated)
    ));
    output.push_str(&format!("Redacted: {}\n", yes_no(request.preview.redacted)));

    if let Some(max_duration_seconds) = request.max_duration_seconds {
        output.push_str(&format!("Max duration: {max_duration_seconds}s\n"));
    }
    if let Some(expires_at) = request.expires_at {
        output.push_str(&format!(
            "Expires at: {}\n",
            format_timestamp_for_prompt(expires_at)
        ));
    }

    output.push_str("Choices: [a]llow [duration_seconds], [d]eny <reason>, abort\n");
    output.push_str("> ");
    output
}

#[must_use]
pub fn render_approval_required_batch(request: &UiApprovalRequest) -> CliOutput {
    CliOutput::failure(
        format!(
            "approval required: {} {} needs {} approval; rerun interactively to approve or deny\n",
            request.resource_kind, request.resource, request.action
        ),
        3,
    )
}

pub fn render_json_approval_frame(frame: UiJsonFrame) -> Result<CliOutput, UiBridgeError> {
    Ok(CliOutput::keep_running(encode_json_frame(&frame)?))
}

fn parse_allow_duration(remaining: Vec<&str>) -> Result<CliApprovalChoice, CliApprovalParseError> {
    match remaining.as_slice() {
        [] => Ok(CliApprovalChoice::Allow {
            lease_duration_seconds: None,
        }),
        [duration] => {
            let duration = duration
                .parse::<u64>()
                .map_err(|_| CliApprovalParseError::InvalidDuration((*duration).to_string()))?;
            Ok(CliApprovalChoice::Allow {
                lease_duration_seconds: Some(duration),
            })
        }
        values => Err(CliApprovalParseError::InvalidDuration(values.join(" "))),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn format_timestamp_for_prompt(timestamp: ruse_types::Timestamp) -> String {
    serde_json::to_string(&timestamp)
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_else(|_| format!("{timestamp:?}"))
}
