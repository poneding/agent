use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{SkillDiscoveryConfig, SkillError, SkillRoot, discover_skills};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManagedSkillRoot {
    pub root_id: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub source_hash: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiscoveredSkillSummary {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub source_path: PathBuf,
    pub source_hash: Option<String>,
    pub token_estimate: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillRootInspection {
    pub root: ManagedSkillRoot,
    pub discovered: Vec<DiscoveredSkillSummary>,
    pub diagnostics: Vec<SkillRootDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillRootDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub skill_id: Option<String>,
}

impl SkillRootDiagnostic {
    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        skill_id: Option<String>,
    ) -> Self {
        Self {
            severity: "error".to_string(),
            code: code.into(),
            message: redact_secret_like_text(&message.into()),
            skill_id,
        }
    }

    fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: "info".to_string(),
            code: code.into(),
            message: redact_secret_like_text(&message.into()),
            skill_id: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ManagedSkillRootRegistry {
    roots: BTreeMap<String, ManagedSkillRoot>,
}

impl ManagedSkillRootRegistry {
    pub fn add(
        &mut self,
        path: impl Into<PathBuf>,
        enabled: bool,
    ) -> Result<ManagedSkillRoot, SkillRootError> {
        let path = path.into();
        if !path.is_dir() {
            return Err(SkillRootError::MissingPath(redact_path(path)));
        }
        let canonical = canonical_path(&path)?;
        if self
            .roots
            .values()
            .filter_map(|root| canonical_path(&root.path).ok())
            .any(|root_path| root_path == canonical)
        {
            return Err(SkillRootError::DuplicatePath(redact_path(path)));
        }

        let root = ManagedSkillRoot {
            root_id: root_id_for_path(&canonical),
            source_hash: source_hash_for_root(&path),
            path,
            enabled,
        };
        self.roots.insert(root.root_id.clone(), root.clone());
        Ok(root)
    }

    #[must_use]
    pub fn list(&self) -> Vec<ManagedSkillRoot> {
        self.roots.values().cloned().collect()
    }

    pub fn inspect(&self, root_id: &str) -> Result<SkillRootInspection, SkillRootError> {
        let root = self
            .roots
            .get(root_id)
            .ok_or_else(|| SkillRootError::UnknownRoot(root_id.to_string()))?;
        Ok(inspect_root(root))
    }

    #[must_use]
    pub fn inspect_all(&self) -> Vec<SkillRootInspection> {
        let mut inspections: Vec<_> = self.roots.values().map(inspect_root).collect();
        add_duplicate_skill_diagnostics(&mut inspections);
        inspections
    }

    pub fn enable(&mut self, root_id: &str) -> Result<ManagedSkillRoot, SkillRootError> {
        let root = self
            .roots
            .get_mut(root_id)
            .ok_or_else(|| SkillRootError::UnknownRoot(root_id.to_string()))?;
        root.enabled = true;
        root.source_hash = source_hash_for_root(&root.path);
        Ok(root.clone())
    }

    pub fn disable(&mut self, root_id: &str) -> Result<ManagedSkillRoot, SkillRootError> {
        let root = self
            .roots
            .get_mut(root_id)
            .ok_or_else(|| SkillRootError::UnknownRoot(root_id.to_string()))?;
        root.enabled = false;
        Ok(root.clone())
    }

    pub fn remove(&mut self, root_id: &str) -> Result<ManagedSkillRoot, SkillRootError> {
        self.roots
            .remove(root_id)
            .ok_or_else(|| SkillRootError::UnknownRoot(root_id.to_string()))
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SkillRootError {
    #[error("skill root path does not exist: {0}")]
    MissingPath(PathBuf),
    #[error("skill root path is already registered: {0}")]
    DuplicatePath(PathBuf),
    #[error("skill root not found: {0}")]
    UnknownRoot(String),
    #[error("skill root path cannot be resolved: {0}")]
    InvalidPath(String),
}

fn inspect_root(root: &ManagedSkillRoot) -> SkillRootInspection {
    if !root.enabled {
        return SkillRootInspection {
            root: root.clone(),
            discovered: Vec::new(),
            diagnostics: vec![SkillRootDiagnostic::info(
                "root_disabled",
                format!("skill root {} is disabled", root.root_id),
            )],
        };
    }

    match discover_skills(&SkillDiscoveryConfig {
        roots: vec![SkillRoot {
            path: root.path.clone(),
            enabled: true,
        }],
    }) {
        Ok(skills) => SkillRootInspection {
            root: refreshed_root(root),
            discovered: skills
                .into_iter()
                .map(|skill| DiscoveredSkillSummary {
                    skill_id: skill.manifest.id.as_str().to_string(),
                    name: skill.manifest.name,
                    description: skill.manifest.description,
                    version: skill.manifest.version,
                    source_path: skill.source.manifest_path.clone(),
                    source_hash: source_hash_for_file(&skill.source.manifest_path),
                    token_estimate: skill.prompt.token_estimate,
                })
                .collect(),
            diagnostics: Vec::new(),
        },
        Err(error) => SkillRootInspection {
            root: refreshed_root(root),
            discovered: Vec::new(),
            diagnostics: vec![diagnostic_from_skill_error(error)],
        },
    }
}

fn refreshed_root(root: &ManagedSkillRoot) -> ManagedSkillRoot {
    let mut root = root.clone();
    root.source_hash = source_hash_for_root(&root.path);
    root
}

fn diagnostic_from_skill_error(error: SkillError) -> SkillRootDiagnostic {
    match error {
        SkillError::RootNotFound(path) => SkillRootDiagnostic::error(
            "root_not_found",
            format!("skill root not found: {}", path.display()),
            None,
        ),
        SkillError::ManifestNotFound(path) => SkillRootDiagnostic::error(
            "manifest_not_found",
            format!("SKILL.md not found: {}", path.display()),
            None,
        ),
        SkillError::InvalidManifest(message) => {
            SkillRootDiagnostic::error("invalid_manifest", message, None)
        }
        SkillError::InvalidPrompt(message) => {
            SkillRootDiagnostic::error("invalid_prompt", message, None)
        }
        SkillError::Io { path, message } => SkillRootDiagnostic::error(
            "io_error",
            format!("failed to read {}: {message}", path.display()),
            None,
        ),
    }
}

fn add_duplicate_skill_diagnostics(inspections: &mut [SkillRootInspection]) {
    let mut counts = BTreeMap::<String, usize>::new();
    for skill_id in inspections
        .iter()
        .flat_map(|inspection| inspection.discovered.iter())
        .map(|summary| summary.skill_id.clone())
    {
        *counts.entry(skill_id).or_default() += 1;
    }
    let duplicates: BTreeSet<_> = counts
        .into_iter()
        .filter_map(|(skill_id, count)| (count > 1).then_some(skill_id))
        .collect();
    if duplicates.is_empty() {
        return;
    }

    for inspection in inspections {
        for skill_id in inspection
            .discovered
            .iter()
            .map(|summary| summary.skill_id.clone())
            .filter(|skill_id| duplicates.contains(skill_id))
            .collect::<Vec<_>>()
        {
            inspection.diagnostics.push(SkillRootDiagnostic::error(
                "duplicate_skill_id",
                format!("duplicate skill id `{skill_id}` is present in multiple enabled roots"),
                Some(skill_id),
            ));
        }
    }
}

fn canonical_path(path: &Path) -> Result<PathBuf, SkillRootError> {
    path.canonicalize()
        .map_err(|error| SkillRootError::InvalidPath(redact_secret_like_text(&error.to_string())))
}

fn redact_path(path: PathBuf) -> PathBuf {
    PathBuf::from(redact_secret_like_text(&path.display().to_string()))
}

fn root_id_for_path(path: &Path) -> String {
    format!("skill-root-{}", stable_hex_hash(path_key(path).as_bytes()))
}

fn source_hash_for_root(path: &Path) -> Option<String> {
    source_hash_for_file(&path.join("SKILL.md"))
}

fn source_hash_for_file(path: &Path) -> Option<String> {
    std::fs::read(path)
        .ok()
        .map(|bytes| stable_hex_hash(bytes.as_slice()))
}

fn stable_hex_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if is_secret_like(token) {
                "[redacted]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
}
