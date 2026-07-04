use agent_core::TraceEntry;
use agent_desktop::{DesktopError, TraceTimeline, build_trace_timeline};
use agent_types::ConversationId;

use crate::{commands::DesktopCommandError, state::DesktopAppState};

#[tauri::command]
pub async fn load_trace(
    conversation_id: String,
    state: tauri::State<'_, DesktopAppState>,
) -> Result<TraceTimeline, DesktopCommandError> {
    let entries = state
        .trace_entries
        .lock()
        .map_err(|_| {
            DesktopCommandError::new("desktop.trace.lock_failed", "trace cache lock failed")
        })?
        .get(&conversation_id)
        .cloned()
        .unwrap_or_default();

    build_trace_response(conversation_id, entries)
}

pub(crate) fn build_trace_response(
    conversation_id: String,
    entries: Vec<TraceEntry>,
) -> Result<TraceTimeline, DesktopCommandError> {
    let conversation_id = parse_conversation_id(&conversation_id)?;
    build_trace_timeline(conversation_id, entries).map_err(trace_error)
}

fn parse_conversation_id(value: &str) -> Result<ConversationId, DesktopCommandError> {
    value.parse::<ConversationId>().map_err(|_| {
        DesktopCommandError::new(
            "desktop.trace.invalid",
            format!("invalid conversation id: {value}"),
        )
    })
}

fn trace_error(error: DesktopError) -> DesktopCommandError {
    DesktopCommandError::new("desktop.trace.invalid", error.to_string())
}
