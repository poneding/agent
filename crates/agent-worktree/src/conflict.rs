use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::WorktreeError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
    pub content_hash: Option<String>,
}

impl FileChange {
    pub fn new(
        path: impl Into<String>,
        kind: ChangeKind,
        content_hash: Option<String>,
    ) -> Result<Self, WorktreeError> {
        let path = validate_relative_path(path.into())?;
        let content_hash = match content_hash {
            Some(value) if value.trim().is_empty() => {
                return Err(WorktreeError::InvalidRequest(
                    "content hash cannot be empty".to_string(),
                ));
            }
            Some(value) => Some(value.trim().to_string()),
            None => None,
        };

        Ok(Self {
            path,
            kind,
            content_hash,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    BothModified,
    DeleteModify,
    AddAddDifferentContent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    pub path: String,
    pub kind: ConflictKind,
    pub lead_change: FileChange,
    pub subagent_change: FileChange,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConflictReport {
    pub conflicts: Vec<Conflict>,
    pub checked_paths: Vec<String>,
}

pub fn detect_conflicts(
    lead_changes: &[FileChange],
    subagent_changes: &[FileChange],
) -> Result<ConflictReport, WorktreeError> {
    let lead = changes_by_path(lead_changes)?;
    let subagent = changes_by_path(subagent_changes)?;
    let checked_paths = lead
        .keys()
        .chain(subagent.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut conflicts = Vec::new();
    for path in &checked_paths {
        let Some(lead_change) = lead.get(path) else {
            continue;
        };
        let Some(subagent_change) = subagent.get(path) else {
            continue;
        };
        if let Some(kind) = classify_conflict(lead_change, subagent_change) {
            conflicts.push(Conflict {
                path: path.clone(),
                kind,
                lead_change: lead_change.clone(),
                subagent_change: subagent_change.clone(),
            });
        }
    }

    Ok(ConflictReport {
        conflicts,
        checked_paths,
    })
}

fn changes_by_path(changes: &[FileChange]) -> Result<BTreeMap<String, FileChange>, WorktreeError> {
    let mut by_path = BTreeMap::new();
    for change in changes {
        let path = validate_relative_path(change.path.clone())?;
        if by_path.insert(path.clone(), change.clone()).is_some() {
            return Err(WorktreeError::InvalidRequest(format!(
                "duplicate file change path: {path}"
            )));
        }
    }
    Ok(by_path)
}

fn classify_conflict(lead: &FileChange, subagent: &FileChange) -> Option<ConflictKind> {
    match (lead.kind, subagent.kind) {
        (ChangeKind::Added, ChangeKind::Added) if lead.content_hash == subagent.content_hash => {
            None
        }
        (ChangeKind::Added, ChangeKind::Added) => Some(ConflictKind::AddAddDifferentContent),
        (ChangeKind::Deleted, ChangeKind::Deleted) => None,
        (ChangeKind::Deleted, _) | (_, ChangeKind::Deleted) => Some(ConflictKind::DeleteModify),
        (ChangeKind::Modified, ChangeKind::Modified)
        | (ChangeKind::Modified, ChangeKind::Added)
        | (ChangeKind::Added, ChangeKind::Modified) => Some(ConflictKind::BothModified),
    }
}

fn validate_relative_path(value: String) -> Result<String, WorktreeError> {
    let value = value.trim().replace('\\', "/");
    if value.is_empty() {
        return Err(WorktreeError::InvalidRequest(
            "file change path cannot be empty".to_string(),
        ));
    }
    if value.starts_with('/') || value.split('/').any(|segment| segment == "..") {
        return Err(WorktreeError::InvalidRequest(
            "file change path must stay relative to the workspace".to_string(),
        ));
    }
    Ok(value)
}
