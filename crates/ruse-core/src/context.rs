use ruse_context::{
    CompactRequest, CompactResult, PromptLayer, PromptLayerKind, PromptLayerPolicy,
    PromptLayerSensitivity, PromptLayerSource, PromptLayerVisibility, PromptMessage,
    assemble_prompt_bundle, compact_messages,
};
use ruse_storage::EventStore;
use ruse_types::{ConversationId, RuntimeEventPayload};

/// Cap reconstructed tool observations so long shell output cannot blow the
/// next-turn context window. Full observations remain in the event store.
const HISTORY_TOOL_OBSERVATION_CHARS: usize = 4_000;

/// Candidate instruction files loaded into the default coding system layer
/// (first existing file wins, scanned from workspace root).
const WORKSPACE_INSTRUCTION_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "AGENT.md",
    ".agents/instructions.md",
];

#[derive(Clone, Debug, Default)]
pub struct ContextAssembler {
    layers: Vec<PromptLayer>,
}

impl ContextAssembler {
    #[must_use]
    pub fn assemble(&self, user_input: &str) -> Vec<String> {
        self.assemble_with_history(user_input, &[])
    }

    #[must_use]
    pub fn assemble_with_history(
        &self,
        user_input: &str,
        history: &[PromptMessage],
    ) -> Vec<String> {
        self.assemble_messages_with_history(user_input, history)
            .into_iter()
            .map(|message| message.content)
            .collect()
    }

    #[must_use]
    pub fn assemble_messages_with_history(
        &self,
        user_input: &str,
        history: &[PromptMessage],
    ) -> Vec<PromptMessage> {
        let mut messages = assemble_prompt_bundle(self.layers.clone())
            .map(|bundle| bundle.messages)
            .unwrap_or_default();
        messages.extend(history.iter().cloned());
        messages.push(PromptMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });
        messages
    }

    #[must_use]
    pub fn with_layers(layers: Vec<PromptLayer>) -> Self {
        Self { layers }
    }

    /// Load the first workspace instruction file found under `workspace_root`
    /// and attach it as a high-priority project system layer.
    #[must_use]
    pub fn with_workspace_instructions(mut self, workspace_root: &std::path::Path) -> Self {
        if let Some(layer) = load_workspace_instruction_layer(workspace_root) {
            self.layers.push(layer);
        }
        self
    }
}

/// Load AGENTS.md / CLAUDE.md style instructions from a workspace root.
///
/// Returns `None` when no known instruction file exists. Content is truncated
/// for token safety; full file remains on disk for tools to read.
#[must_use]
pub fn load_workspace_instruction_layer(workspace_root: &std::path::Path) -> Option<PromptLayer> {
    const MAX_INSTRUCTION_CHARS: usize = 12_000;
    for relative in WORKSPACE_INSTRUCTION_FILES {
        let path = workspace_root.join(relative);
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let content = if trimmed.chars().count() > MAX_INSTRUCTION_CHARS {
            let mut out = trimmed
                .chars()
                .take(MAX_INSTRUCTION_CHARS)
                .collect::<String>();
            out.push_str("\n\n[truncated workspace instructions]");
            out
        } else {
            trimmed.to_string()
        };
        let token_estimate = (content.len() as u32 / 4).max(1);
        let content_hash = simple_content_hash(&content);
        return PromptLayer::new(
            "workspace_instructions",
            PromptLayerKind::Project,
            900,
            content,
            token_estimate,
            PromptLayerSource::ProjectFile {
                path: relative.to_string(),
                content_hash,
            },
            PromptLayerVisibility::Full,
            PromptLayerSensitivity::Workspace,
            PromptLayerPolicy::Optional,
            1,
        )
        .ok();
    }
    None
}

/// Compact git/repo status summary for the system layer (best-effort, no network).
#[must_use]
pub fn load_workspace_git_summary_layer(workspace_root: &std::path::Path) -> Option<PromptLayer> {
    let output = std::process::Command::new("git")
        .args(["status", "--short", "--branch"])
        .current_dir(workspace_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let content = if trimmed.lines().count() > 40 {
        let mut lines = trimmed.lines().take(40).collect::<Vec<_>>();
        lines.push("… [git status truncated]");
        lines.join("\n")
    } else {
        trimmed.to_string()
    };
    let body = format!("Current git status:\n{content}");
    let token_estimate = (body.len() as u32 / 4).max(1);
    PromptLayer::new(
        "workspace_git_status",
        PromptLayerKind::Project,
        800,
        body,
        token_estimate,
        PromptLayerSource::BuiltIn,
        PromptLayerVisibility::SummaryOnly,
        PromptLayerSensitivity::Workspace,
        PromptLayerPolicy::Optional,
        1,
    )
    .ok()
}

/// Build the default coding-agent context assembler for a workspace root.
#[must_use]
pub fn coding_context_assembler(workspace_root: &std::path::Path) -> ContextAssembler {
    let mut layers = Vec::new();
    if let Some(instructions) = load_workspace_instruction_layer(workspace_root) {
        layers.push(instructions);
    }
    if let Some(git) = load_workspace_git_summary_layer(workspace_root) {
        layers.push(git);
    }
    ContextAssembler::with_layers(layers)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationCompactionInput {
    pub conversation_id: ConversationId,
    pub max_summary_tokens: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationCompactionReport {
    pub conversation_id: ConversationId,
    pub source_message_count: u32,
    pub summary_message_count: u32,
    pub dropped_message_count: u32,
    pub estimated_tokens: u32,
    pub summary: Option<String>,
}

pub fn compact_conversation_events<E>(
    store: &E,
    input: ConversationCompactionInput,
) -> Result<ConversationCompactionReport, String>
where
    E: EventStore + ?Sized,
{
    let events = store
        .list_by_conversation(input.conversation_id)
        .map_err(|error| error.to_string())?;
    let messages = events
        .iter()
        .filter_map(|stored| prompt_message_from_payload(&stored.event.payload))
        .collect::<Vec<_>>();

    let source_message_count = usize_to_u32(messages.len());
    let result = compact_messages(CompactRequest {
        messages,
        pinned: Vec::new(),
        max_summary_tokens: input.max_summary_tokens,
    })
    .map_err(|error| error.to_string())?;

    Ok(compaction_report(
        input.conversation_id,
        source_message_count,
        result,
    ))
}

pub fn conversation_history_messages<E>(
    store: &E,
    conversation_id: ConversationId,
    max_messages: usize,
) -> Result<Vec<PromptMessage>, String>
where
    E: EventStore + ?Sized,
{
    if max_messages == 0 {
        return Ok(Vec::new());
    }

    let events = store
        .list_by_conversation(conversation_id)
        .map_err(|error| error.to_string())?;
    let mut messages =
        reconstruct_prompt_history(events.iter().map(|stored| &stored.event.payload));
    let overflow = messages.len().saturating_sub(max_messages);
    if overflow > 0 {
        messages.drain(0..overflow);
    }
    Ok(messages)
}

/// Reconstruct multi-turn prompt history including tool calls/results so a
/// restarted session does not lose tool-augmented continuity.
fn reconstruct_prompt_history<'a, I>(payloads: I) -> Vec<PromptMessage>
where
    I: IntoIterator<Item = &'a RuntimeEventPayload>,
{
    let mut messages = Vec::new();
    let mut pending_tool_calls: Vec<(String, String, String)> = Vec::new();

    let flush_tools = |pending: &mut Vec<(String, String, String)>,
                       out: &mut Vec<PromptMessage>| {
        if pending.is_empty() {
            return;
        }
        // Narrative form — not a wire protocol. Models that saw
        // `tool_call id=… name=… input=…` as assistant text would imitate it
        // as prose and the TUI would show raw dumps instead of running tools.
        let body = pending
            .iter()
            .map(|(call_id, name, input)| {
                format!("[used tool {name} (call_id={call_id})]\narguments: {input}")
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        out.push(PromptMessage {
            role: "assistant".to_string(),
            content: redact_secret_like_text(&body),
        });
        pending.clear();
    };

    for payload in payloads {
        match payload {
            RuntimeEventPayload::UserInputReceived(input) => {
                flush_tools(&mut pending_tool_calls, &mut messages);
                messages.push(PromptMessage {
                    role: "user".to_string(),
                    content: redact_secret_like_text(&input.message),
                });
            }
            RuntimeEventPayload::FinalResponseProduced(response) => {
                flush_tools(&mut pending_tool_calls, &mut messages);
                if response.message.trim().is_empty() {
                    continue;
                }
                messages.push(PromptMessage {
                    role: "assistant".to_string(),
                    content: redact_secret_like_text(&response.message),
                });
            }
            RuntimeEventPayload::ToolCallProposed(call) => {
                let input = compact_json(&call.input_json, 500);
                pending_tool_calls.push((call.call_id.clone(), call.tool_name.clone(), input));
            }
            RuntimeEventPayload::ToolObservationRecorded(observed) => {
                flush_tools(&mut pending_tool_calls, &mut messages);
                let observation =
                    truncate_chars(&observed.observation, HISTORY_TOOL_OBSERVATION_CHARS);
                messages.push(PromptMessage {
                    role: "user".to_string(),
                    content: redact_secret_like_text(&format!(
                        "[tool result call_id={}]\n{observation}",
                        observed.call_id
                    )),
                });
            }
            RuntimeEventPayload::ToolExecutionFailed(failed) => {
                flush_tools(&mut pending_tool_calls, &mut messages);
                messages.push(PromptMessage {
                    role: "user".to_string(),
                    content: redact_secret_like_text(&format!(
                        "[tool error call_id={} code={}]\n{}",
                        failed.call_id, failed.error_code, failed.message
                    )),
                });
            }
            _ => {}
        }
    }
    flush_tools(&mut pending_tool_calls, &mut messages);
    messages
}

fn prompt_message_from_payload(payload: &RuntimeEventPayload) -> Option<PromptMessage> {
    match payload {
        RuntimeEventPayload::UserInputReceived(input) => Some(PromptMessage {
            role: "user".to_string(),
            content: redact_secret_like_text(&input.message),
        }),
        RuntimeEventPayload::FinalResponseProduced(response) => Some(PromptMessage {
            role: "assistant".to_string(),
            content: redact_secret_like_text(&response.message),
        }),
        _ => None,
    }
}

fn compact_json(value: &serde_json::Value, max_chars: usize) -> String {
    let raw = value.to_string();
    truncate_chars(&raw, max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut out = value.chars().take(max_chars).collect::<String>();
    out.push('…');
    out
}

fn simple_content_hash(content: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn compaction_report(
    conversation_id: ConversationId,
    source_message_count: u32,
    result: CompactResult,
) -> ConversationCompactionReport {
    let (summary, estimated_tokens, summary_message_count) = result
        .summary
        .map(|summary| {
            (
                Some(redact_secret_like_text(&summary.text)),
                summary.estimated_tokens,
                1,
            )
        })
        .unwrap_or((None, 0, 0));

    ConversationCompactionReport {
        conversation_id,
        source_message_count,
        summary_message_count,
        dropped_message_count: usize_to_u32(result.dropped_message_count),
        estimated_tokens,
        summary,
    }
}

fn usize_to_u32(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if lower.contains("secret")
                || lower.contains("token")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("password")
                || lower.starts_with("sk-")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruse_types::{
        FinalResponseProduced, ToolCallProposed, ToolObservationRecorded, UserInputReceived,
    };
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn reconstructs_tool_calls_and_observations_in_order() {
        let payloads = [
            RuntimeEventPayload::UserInputReceived(UserInputReceived {
                message: "edit notes".to_string(),
            }),
            RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
                call_id: "c1".to_string(),
                tool_name: "workspace.edit_file".to_string(),
                input_json: json!({"path": "notes.txt"}),
            }),
            RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
                call_id: "c1".to_string(),
                observation: "ok".to_string(),
            }),
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: "done".to_string(),
            }),
        ];
        let history = reconstruct_prompt_history(payloads.iter());
        assert_eq!(history.len(), 4);
        assert_eq!(history[0].role, "user");
        assert_eq!(history[1].role, "assistant");
        assert!(history[1].content.contains("workspace.edit_file"));
        assert_eq!(history[2].role, "user");
        assert!(history[2].content.contains("tool result"));
        assert_eq!(history[3].content, "done");
    }

    #[test]
    fn loads_agents_md_as_workspace_instruction_layer() {
        let temp = tempdir().expect("temp");
        fs::write(temp.path().join("AGENTS.md"), "# rules\n- be concise\n").expect("write");
        let layer = load_workspace_instruction_layer(temp.path()).expect("layer");
        assert!(layer.content.contains("be concise"));
        assert_eq!(layer.id.as_str(), "workspace_instructions");
    }

    #[test]
    fn coding_assembler_includes_instruction_layer() {
        let temp = tempdir().expect("temp");
        fs::write(temp.path().join("CLAUDE.md"), "use rustfmt").expect("write");
        let assembler = coding_context_assembler(temp.path());
        let messages = assembler.assemble_messages_with_history("hi", &[]);
        assert!(
            messages.iter().any(|m| m.content.contains("use rustfmt")),
            "expected CLAUDE.md content in assembled messages: {messages:?}"
        );
    }
}
