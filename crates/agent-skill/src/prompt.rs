use std::path::PathBuf;

use agent_context::{PromptLayer, PromptLayerKind};
use serde::{Deserialize, Serialize};

use crate::SkillId;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillPromptContribution {
    pub skill_id: SkillId,
    pub layer_id: String,
    pub content: String,
    pub priority: i32,
    pub token_estimate: u32,
    pub source_path: PathBuf,
}

#[must_use]
pub fn skill_prompt_layer(contribution: &SkillPromptContribution) -> PromptLayer {
    PromptLayer {
        id: contribution.layer_id.clone(),
        kind: PromptLayerKind::Skill,
        priority: contribution.priority,
        content: contribution.content.clone(),
        token_estimate: contribution.token_estimate,
    }
}
