use agent_core::ReleaseChannel;
use serde::{Deserialize, Serialize};

use crate::DesktopError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub channel: ReleaseChannel,
    pub version: String,
    pub artifacts: Vec<DesktopUpdateArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopUpdateArtifact {
    pub platform: String,
    pub url: String,
    pub checksum_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DesktopUpdateDecision {
    UpToDate {
        current_version: String,
    },
    UpdateAvailable {
        current_version: String,
        target_version: String,
        artifact_count: usize,
    },
}

pub fn parse_update_manifest_json(input: &str) -> Result<UpdateManifest, DesktopError> {
    let manifest: UpdateManifest = serde_json::from_str(input)
        .map_err(|error| invalid_update_manifest(format!("manifest JSON is invalid: {error}")))?;
    validate_update_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_update_manifest(manifest: &UpdateManifest) -> Result<(), DesktopError> {
    if manifest.version.trim().is_empty() {
        return Err(invalid_update_manifest("version cannot be empty"));
    }
    if manifest.artifacts.is_empty() {
        return Err(invalid_update_manifest(
            "at least one update artifact is required",
        ));
    }

    for (index, artifact) in manifest.artifacts.iter().enumerate() {
        validate_artifact(index, artifact)?;
    }

    Ok(())
}

pub fn validate_update_manifest_for_current(
    current_version: impl Into<String>,
    manifest: UpdateManifest,
) -> Result<DesktopUpdateDecision, DesktopError> {
    let current_version = current_version.into();
    if current_version.trim().is_empty() {
        return Err(invalid_update_manifest("current version cannot be empty"));
    }
    validate_update_manifest(&manifest)?;

    if manifest.version == current_version {
        Ok(DesktopUpdateDecision::UpToDate { current_version })
    } else {
        Ok(DesktopUpdateDecision::UpdateAvailable {
            current_version,
            target_version: manifest.version,
            artifact_count: manifest.artifacts.len(),
        })
    }
}

fn validate_artifact(index: usize, artifact: &DesktopUpdateArtifact) -> Result<(), DesktopError> {
    let prefix = format!("artifacts[{index}]");
    if artifact.platform.trim().is_empty() {
        return Err(invalid_update_manifest(format!(
            "{prefix}.platform cannot be empty"
        )));
    }
    if !is_supported_url(&artifact.url) {
        return Err(invalid_update_manifest(format!(
            "{prefix}.url must use file, http, or https"
        )));
    }
    if !is_sha256_hex(&artifact.checksum_sha256) {
        return Err(invalid_update_manifest(format!(
            "{prefix}.checksum_sha256 must be 64 lowercase hex characters"
        )));
    }
    if artifact
        .signature
        .as_ref()
        .is_some_and(|signature| signature.trim().is_empty())
    {
        return Err(invalid_update_manifest(format!(
            "{prefix}.signature cannot be empty when present"
        )));
    }

    Ok(())
}

fn is_supported_url(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("file://") || value.starts_with("https://") || value.starts_with("http://")
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid_update_manifest(message: impl Into<String>) -> DesktopError {
    DesktopError::InvalidUpdateManifest(message.into())
}
