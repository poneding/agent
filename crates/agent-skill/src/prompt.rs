use std::path::PathBuf;

use agent_context::{
    PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility,
};
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
    PromptLayer::new(
        contribution.layer_id.clone(),
        PromptLayerKind::Skill,
        contribution.priority,
        contribution.content.clone(),
        contribution.token_estimate,
        PromptLayerSource::Skill {
            skill_id: contribution.skill_id.as_str().to_string(),
            source: contribution.source_path.display().to_string(),
        },
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::Workspace,
        PromptLayerPolicy::Optional,
        1,
    )
    .expect("validated skill prompt contribution")
}
