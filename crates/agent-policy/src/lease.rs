use agent_types::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::decision::PolicyDecision;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PermissionLease {
    pub id: String,
    pub request_id: String,
    pub decision: PolicyDecision,
    pub issued_at: Timestamp,
}

impl PermissionLease {
    #[must_use]
    pub fn new(request_id: impl Into<String>, decision: PolicyDecision) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            request_id: request_id.into(),
            decision,
            issued_at: Timestamp::now_utc(),
        }
    }
}
