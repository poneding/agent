use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::SkillError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SkillId(String);

impl SkillId {
    pub fn new(value: impl Into<String>) -> Result<Self, SkillError> {
        let value = value.into();
        if value.is_empty()
            || !value.chars().all(|ch| {
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.')
            })
        {
            return Err(SkillError::InvalidManifest(format!(
                "invalid skill id `{value}`"
            )));
        }

        Ok(Self(value))
    }

    pub fn from_name(name: &str) -> Result<Self, SkillError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(SkillError::InvalidManifest(
                "skill name cannot be empty".to_string(),
            ));
        }

        let mut has_alphanumeric = false;
        let mut previous_dash = false;
        let mut value = String::new();

        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                has_alphanumeric = true;
                previous_dash = false;
                value.push(ch.to_ascii_lowercase());
            } else if !previous_dash {
                previous_dash = true;
                value.push('-');
            }
        }

        let value = value.trim_matches('-').to_string();
        if !has_alphanumeric || value.is_empty() {
            return Err(SkillError::InvalidManifest(format!(
                "skill name `{name}` must contain an ASCII letter or digit"
            )));
        }

        Self::new(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SkillId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SkillId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
}

impl SkillManifest {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        version: Option<String>,
    ) -> Result<Self, SkillError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(SkillError::InvalidManifest(
                "skill name cannot be empty".to_string(),
            ));
        }

        let description = description.into();
        if description.trim().is_empty() {
            return Err(SkillError::InvalidManifest(
                "skill description cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: SkillId::from_name(&name)?,
            name,
            description,
            version,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SkillFrontMatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

impl SkillFrontMatter {
    pub fn into_manifest(self) -> Result<SkillManifest, SkillError> {
        let name = self.name.ok_or_else(|| {
            SkillError::InvalidManifest("missing required field `name`".to_string())
        })?;
        let description = self.description.ok_or_else(|| {
            SkillError::InvalidManifest("missing required field `description`".to_string())
        })?;

        SkillManifest::new(name, description, self.version)
    }
}
