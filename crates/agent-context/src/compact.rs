use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{ContextError, PromptMessage};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedMessage {
    pub id: String,
    pub reason: String,
    pub message: PromptMessage,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactionSummary {
    pub text: String,
    pub source_message_count: usize,
    pub estimated_tokens: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactRequest {
    pub messages: Vec<PromptMessage>,
    pub pinned: Vec<PinnedMessage>,
    pub max_summary_tokens: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactResult {
    pub pinned: Vec<PinnedMessage>,
    pub summary: Option<CompactionSummary>,
    pub dropped_message_count: usize,
}

pub fn compact_messages(request: CompactRequest) -> Result<CompactResult, ContextError> {
    validate_pins(&request.pinned)?;
    if !request.messages.is_empty() && request.max_summary_tokens == 0 {
        return Err(ContextError::InvalidCompaction(
            "max_summary_tokens must be greater than zero when messages are compacted".to_string(),
        ));
    }

    if request.messages.is_empty() {
        return Ok(CompactResult {
            pinned: request.pinned,
            summary: None,
            dropped_message_count: 0,
        });
    }

    let source_message_count = request.messages.len();
    let text = summary_text(&request.messages, request.max_summary_tokens);
    let estimated_tokens = estimate_tokens(&text);

    Ok(CompactResult {
        pinned: request.pinned,
        summary: Some(CompactionSummary {
            text,
            source_message_count,
            estimated_tokens,
        }),
        dropped_message_count: source_message_count,
    })
}

fn validate_pins(pinned: &[PinnedMessage]) -> Result<(), ContextError> {
    let mut ids = BTreeSet::new();
    for pin in pinned {
        if pin.id.trim().is_empty() {
            return Err(ContextError::InvalidPin(
                "pin id cannot be empty".to_string(),
            ));
        }
        if pin.reason.trim().is_empty() {
            return Err(ContextError::InvalidPin(format!(
                "pin {} reason cannot be empty",
                pin.id
            )));
        }
        if !ids.insert(pin.id.as_str()) {
            return Err(ContextError::InvalidPin(format!(
                "duplicate pin id {}",
                pin.id
            )));
        }
    }
    Ok(())
}

fn summary_text(messages: &[PromptMessage], max_summary_tokens: u32) -> String {
    let full_text = messages
        .iter()
        .map(|message| format!("{}: {}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n");
    let tokens = full_text.split_whitespace().collect::<Vec<_>>();

    if tokens.len() <= max_summary_tokens as usize {
        return full_text;
    }

    tokens
        .into_iter()
        .take(max_summary_tokens as usize)
        .collect::<Vec<_>>()
        .join(" ")
}

fn estimate_tokens(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}
