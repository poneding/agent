use std::collections::BTreeMap;

use crate::capability::{CapabilityAction, ResourceScope};
use crate::decision::PolicyDecision;
use crate::error::PolicyError;
use crate::lease::PermissionLease;
use crate::request::PermissionRequest;

pub trait PolicyEngine: Send + Sync {
    fn decide(&self, request: &PermissionRequest) -> Result<PolicyDecision, PolicyError>;

    fn issue_lease(
        &self,
        request: &PermissionRequest,
        decision: PolicyDecision,
    ) -> Result<Option<PermissionLease>, PolicyError>;
}

#[derive(Clone, Debug, Default)]
pub struct StaticPolicyEngine {
    rules: BTreeMap<PolicyRuleKey, PolicyDecision>,
}

impl StaticPolicyEngine {
    #[must_use]
    pub fn with_decision(
        mut self,
        action: CapabilityAction,
        scope: ResourceScope,
        decision: PolicyDecision,
    ) -> Self {
        self.rules.insert(PolicyRuleKey { action, scope }, decision);
        self
    }
}

impl PolicyEngine for StaticPolicyEngine {
    fn decide(&self, request: &PermissionRequest) -> Result<PolicyDecision, PolicyError> {
        Ok(self
            .rules
            .get(&PolicyRuleKey {
                action: request.action,
                scope: request.scope.clone(),
            })
            .cloned()
            .unwrap_or_else(|| PolicyDecision::Ask {
                reason: "no matching policy rule".to_string(),
            }))
    }

    fn issue_lease(
        &self,
        request: &PermissionRequest,
        decision: PolicyDecision,
    ) -> Result<Option<PermissionLease>, PolicyError> {
        if matches!(decision, PolicyDecision::Allow { .. }) {
            return Ok(Some(PermissionLease::new(request.id.clone(), decision)));
        }

        Ok(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct PolicyRuleKey {
    action: CapabilityAction,
    scope: ResourceScope,
}
