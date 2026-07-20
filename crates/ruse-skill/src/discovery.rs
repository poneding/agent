use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::manifest::SkillFrontMatter;
use crate::{SkillError, SkillManifest, SkillPromptContribution};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillRoot {
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillDiscoveryConfig {
    pub roots: Vec<SkillRoot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillSource {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiscoveredSkill {
    pub manifest: SkillManifest,
    pub source: SkillSource,
    pub prompt: SkillPromptContribution,
}

pub fn discover_skills(config: &SkillDiscoveryConfig) -> Result<Vec<DiscoveredSkill>, SkillError> {
    let mut roots: Vec<_> = config.roots.iter().filter(|root| root.enabled).collect();
    roots.sort_by_key(|root| path_key(&root.path));

    roots
        .into_iter()
        .map(|root| discover_skill_root(&root.path))
        .collect()
}

fn discover_skill_root(root_path: &Path) -> Result<DiscoveredSkill, SkillError> {
    if !root_path.is_dir() {
        return Err(SkillError::RootNotFound(root_path.to_path_buf()));
    }

    let manifest_path = root_path.join("SKILL.md");
    if !manifest_path.is_file() {
        return Err(SkillError::ManifestNotFound(manifest_path));
    }

    let content = std::fs::read_to_string(&manifest_path).map_err(|err| SkillError::Io {
        path: manifest_path.clone(),
        message: err.to_string(),
    })?;

    let (manifest, body) = parse_skill_markdown(&content)?;
    let content = body.trim().to_string();
    if content.is_empty() {
        return Err(SkillError::InvalidPrompt(
            "skill prompt body cannot be empty".to_string(),
        ));
    }

    let prompt = SkillPromptContribution {
        skill_id: manifest.id.clone(),
        layer_id: format!("skill:{}", manifest.id.as_str()),
        token_estimate: estimate_tokens(&content),
        content,
        priority: 600,
        source_path: manifest_path.clone(),
    };

    Ok(DiscoveredSkill {
        manifest,
        source: SkillSource {
            root_path: root_path.to_path_buf(),
            manifest_path,
        },
        prompt,
    })
}

fn parse_skill_markdown(content: &str) -> Result<(SkillManifest, String), SkillError> {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return Err(SkillError::InvalidManifest(
            "SKILL.md must start with YAML front matter".to_string(),
        ));
    }

    let mut yaml_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut found_end = false;

    for line in lines {
        if !found_end && line == "---" {
            found_end = true;
            continue;
        }

        if found_end {
            body_lines.push(line);
        } else {
            yaml_lines.push(line);
        }
    }

    if !found_end {
        return Err(SkillError::InvalidManifest(
            "YAML front matter must end with ---".to_string(),
        ));
    }

    let front_matter: SkillFrontMatter = serde_yaml::from_str(&yaml_lines.join("\n"))
        .map_err(|err| SkillError::InvalidManifest(err.to_string()))?;
    let manifest = front_matter.into_manifest()?;

    Ok((manifest, body_lines.join("\n")))
}

fn estimate_tokens(content: &str) -> u32 {
    content.split_whitespace().count().max(1) as u32
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
