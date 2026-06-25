use agent_types::{ConversationId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::DesktopError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopConversationItem {
    pub conversation_id: ConversationId,
    pub title: String,
    pub last_activity_at: Timestamp,
    pub unread_count: u32,
}

impl DesktopConversationItem {
    pub fn new(
        conversation_id: ConversationId,
        title: impl Into<String>,
        last_activity_at: Timestamp,
        unread_count: u32,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            conversation_id,
            title: validate_non_empty("conversation title", title.into())?,
            last_activity_at,
            unread_count,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("conversation title", self.title.clone())?;
        Ok(())
    }
}

pub(crate) fn validate_non_empty(label: &str, value: String) -> Result<String, DesktopError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopError::InvalidSnapshot(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}
