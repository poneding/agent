use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ToolNameError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ToolName(String);

impl ToolName {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolNameError> {
        let value = value.into();

        if value.is_empty() {
            return Err(ToolNameError::Empty);
        }

        if !value.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.')
        }) {
            return Err(ToolNameError::InvalidCharacters(value));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ToolName {
    type Err = ToolNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for ToolName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ToolName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: ToolName,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolInput {
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolOutput {
    pub value: serde_json::Value,
}
