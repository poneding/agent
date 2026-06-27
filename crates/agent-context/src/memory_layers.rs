use agent_memory::{MemorySelectedCandidate, MemorySensitivity};

use crate::{
    ContextError, PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity,
    PromptLayerSource, PromptLayerVisibility,
};

const MEMORY_LAYER_BASE_PRIORITY: i32 = 40;

pub fn memory_prompt_layers(
    selected: &[MemorySelectedCandidate],
) -> Result<Vec<PromptLayer>, ContextError> {
    selected.iter().map(memory_prompt_layer).collect()
}

pub fn memory_prompt_layer(
    selected: &MemorySelectedCandidate,
) -> Result<PromptLayer, ContextError> {
    let record = &selected.candidate.record;
    let (visibility, sensitivity) = memory_prompt_privacy(record.sensitivity);
    let priority = MEMORY_LAYER_BASE_PRIORITY.saturating_sub(rank_as_i32(selected.rank));

    PromptLayer::new(
        format!("memory:{}", record.id),
        PromptLayerKind::Memory,
        priority,
        record.text.clone(),
        selected.token_estimate,
        PromptLayerSource::Memory {
            memory_id: record.id.clone(),
        },
        visibility,
        sensitivity,
        PromptLayerPolicy::Optional,
        1,
    )
}

const fn memory_prompt_privacy(
    sensitivity: MemorySensitivity,
) -> (PromptLayerVisibility, PromptLayerSensitivity) {
    match sensitivity {
        MemorySensitivity::Public => (PromptLayerVisibility::Full, PromptLayerSensitivity::Public),
        MemorySensitivity::Internal => (
            PromptLayerVisibility::SummaryOnly,
            PromptLayerSensitivity::Workspace,
        ),
        MemorySensitivity::Sensitive => (
            PromptLayerVisibility::Hidden,
            PromptLayerSensitivity::UserPrivate,
        ),
    }
}

fn rank_as_i32(rank: usize) -> i32 {
    i32::try_from(rank).unwrap_or(i32::MAX)
}
