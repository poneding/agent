use std::collections::BTreeSet;

use ruse_policy::{CapabilityAction, ResourceScope};
use serde::{Deserialize, Serialize};

use crate::TeamError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub action: CapabilityAction,
    pub scope: ResourceScope,
}

impl CapabilityGrant {
    pub fn new(action: CapabilityAction, scope: ResourceScope) -> Result<Self, TeamError> {
        if scope.pattern.trim().is_empty() {
            return Err(TeamError::InvalidHandoff(
                "capability scope pattern cannot be empty".to_string(),
            ));
        }

        Ok(Self { action, scope })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub grants: Vec<CapabilityGrant>,
}

impl CapabilitySet {
    pub fn new(grants: Vec<CapabilityGrant>) -> Result<Self, TeamError> {
        let set = Self::from_grants(grants);
        if set.grants.is_empty() {
            return Err(TeamError::InvalidHandoff(
                "capability set cannot be empty".to_string(),
            ));
        }

        Ok(set)
    }

    #[must_use]
    pub fn contains(&self, grant: &CapabilityGrant) -> bool {
        self.grants.contains(grant)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    fn from_grants(grants: Vec<CapabilityGrant>) -> Self {
        let grants = grants
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self { grants }
    }
}

#[must_use]
pub fn reduce_capabilities(
    parent: &CapabilitySet,
    role: &CapabilitySet,
    task: &CapabilitySet,
) -> CapabilitySet {
    CapabilitySet::from_grants(
        parent
            .grants
            .iter()
            .filter(|grant| role.contains(grant) && task.contains(grant))
            .cloned()
            .collect(),
    )
}
