#![forbid(unsafe_code)]
//! Skill discovery and prompt contribution contracts for agent.

pub mod discovery;
pub mod error;
pub mod manifest;
pub mod prompt;
pub mod roots;

pub use discovery::{
    DiscoveredSkill, SkillDiscoveryConfig, SkillRoot, SkillSource, discover_skills,
};
pub use error::SkillError;
pub use manifest::{SkillId, SkillManifest};
pub use prompt::{SkillPromptContribution, skill_prompt_layer};
pub use roots::{
    DiscoveredSkillSummary, ManagedSkillRoot, ManagedSkillRootRegistry, SkillRootDiagnostic,
    SkillRootError, SkillRootInspection,
};
