use serde::{Deserialize, Serialize};

use crate::error::AgentTypesError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const CURRENT_VALUE: u16 = 1;

    #[must_use]
    pub const fn current() -> Self {
        Self(Self::CURRENT_VALUE)
    }

    pub fn new(value: u16) -> Result<Self, AgentTypesError> {
        if value == 0 {
            return Err(AgentTypesError::InvalidSchemaVersion(value));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_schema_version_is_one() {
        assert_eq!(SchemaVersion::current().get(), 1);
    }

    #[test]
    fn schema_version_rejects_zero() {
        let error = SchemaVersion::new(0).expect_err("zero schema version");
        assert_eq!(error.to_string(), "schema version must be positive: 0");
    }
}
