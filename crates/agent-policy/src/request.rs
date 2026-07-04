use agent_types::{ActorRef, CorrelationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capability::{CapabilityAction, ResourceScope};
use crate::error::PolicyError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub actor: ActorRef,
    pub action: CapabilityAction,
    pub scope: ResourceScope,
    pub reason: String,
    pub correlation_id: CorrelationId,
}

impl PermissionRequest {
    pub fn new(
        actor: ActorRef,
        action: CapabilityAction,
        scope: ResourceScope,
        reason: impl Into<String>,
        correlation_id: CorrelationId,
    ) -> Result<Self, PolicyError> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(PolicyError::InvalidRequest(
                "permission request reason cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            actor,
            action,
            scope,
            reason,
            correlation_id,
        })
    }
}
