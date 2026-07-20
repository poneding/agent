use serde::{Deserialize, Serialize};

use crate::DesktopError;
use ruse_ui_bridge::UiExtensionSnapshot;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopExtensionStatus {
    Healthy,
    NeedsAttention,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopExtensionHealth {
    pub status: DesktopExtensionStatus,
    pub total_count: u32,
    pub enabled_count: u32,
    pub unhealthy_count: u32,
    pub review_count: u32,
    pub snapshot: UiExtensionSnapshot,
}

impl DesktopExtensionHealth {
    pub fn new(snapshot: UiExtensionSnapshot) -> Result<Self, DesktopError> {
        for extension in &snapshot.extensions {
            validate_extension_id(&extension.id)?;
        }

        let total_count = snapshot.extensions.len() as u32;
        let enabled_count = snapshot
            .extensions
            .iter()
            .filter(|extension| extension.enabled)
            .count() as u32;
        let unhealthy_count = snapshot
            .extensions
            .iter()
            .filter(|extension| extension.health != "healthy")
            .count() as u32;
        let review_count = snapshot.capability_reviews.len() as u32;
        let status = if unhealthy_count == 0 && review_count == 0 && snapshot.diagnostics.is_empty()
        {
            DesktopExtensionStatus::Healthy
        } else {
            DesktopExtensionStatus::NeedsAttention
        };

        Ok(Self {
            status,
            total_count,
            enabled_count,
            unhealthy_count,
            review_count,
            snapshot,
        })
    }
}

fn validate_extension_id(value: &str) -> Result<(), DesktopError> {
    if value.trim().is_empty() {
        return Err(DesktopError::InvalidSnapshot(
            "extension id cannot be empty".to_string(),
        ));
    }
    Ok(())
}
