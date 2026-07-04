use std::fmt;
use std::str::FromStr;

use agent_types::Timestamp;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::event_store::StorageError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IdempotencyKey(String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolEffectRecord {
    pub key: IdempotencyKey,
    pub call_id: String,
    pub tool_name: String,
    pub input_hash: String,
    pub output_json: serde_json::Value,
    pub completed_at: Timestamp,
}

pub trait IdempotencyStore {
    fn record_effect(&mut self, record: ToolEffectRecord) -> Result<(), StorageError>;

    fn get_effect(&self, key: &IdempotencyKey) -> Result<Option<ToolEffectRecord>, StorageError>;
}

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, StorageError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(StorageError::InvalidIdempotencyKey(
                "idempotency key cannot be empty".to_string(),
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for IdempotencyKey {
    type Err = StorageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for IdempotencyKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IdempotencyKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

pub(crate) fn validate_effect(record: &ToolEffectRecord) -> Result<(), StorageError> {
    if record.call_id.trim().is_empty() {
        return Err(StorageError::InvalidIdempotencyRecord(
            "call_id cannot be empty".to_string(),
        ));
    }
    if record.tool_name.trim().is_empty() {
        return Err(StorageError::InvalidIdempotencyRecord(
            "tool_name cannot be empty".to_string(),
        ));
    }
    if record.input_hash.trim().is_empty() {
        return Err(StorageError::InvalidIdempotencyRecord(
            "input_hash cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn conflict(key: &IdempotencyKey, existing_hash: &str, new_hash: &str) -> StorageError {
    StorageError::IdempotencyConflict {
        key: key.to_string(),
        existing_hash: existing_hash.to_string(),
        new_hash: new_hash.to_string(),
    }
}
