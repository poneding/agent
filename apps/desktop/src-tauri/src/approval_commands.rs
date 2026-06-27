use agent_types::{ApprovalRequestId, ClientKind, CorrelationId, SchemaVersion};
use agent_ui_bridge::{UiApprovalDecision, UiClientInfo, UiCommand, UiCommandEnvelope};
use serde::{Deserialize, Serialize};

use crate::bridge::{DesktopCommandAck, DesktopRuntimeSession};
use crate::commands::DesktopCommandError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopResolveApprovalRequest {
    pub request_id: String,
    pub decision: UiApprovalDecision,
    pub reason: String,
    pub lease_duration_seconds: Option<u64>,
}

impl DesktopResolveApprovalRequest {
    pub fn to_ui_command_envelope(
        &self,
        client_version: impl Into<String>,
    ) -> Result<UiCommandEnvelope, DesktopCommandError> {
        let reason = self.reason.trim().to_string();
        if reason.is_empty() {
            return Err(DesktopCommandError::new(
                "desktop.approval.invalid",
                "resolve approval reason cannot be empty",
            ));
        }

        let request_id = self.request_id.parse::<ApprovalRequestId>().map_err(|_| {
            DesktopCommandError::new(
                "desktop.approval.invalid",
                format!("invalid approval request id: {}", self.request_id),
            )
        })?;

        Ok(UiCommandEnvelope {
            schema_version: SchemaVersion::current(),
            command_id: CorrelationId::new(),
            client: UiClientInfo {
                kind: ClientKind::Desktop,
                version: client_version.into(),
            },
            command: UiCommand::ResolveApproval {
                request_id,
                decision: self.decision,
                reason,
                lease_duration_seconds: self.lease_duration_seconds,
            },
        })
    }
}

pub fn resolve_approval_in_session(
    session: &mut DesktopRuntimeSession,
    request: DesktopResolveApprovalRequest,
    client_version: impl Into<String>,
) -> Result<DesktopCommandAck, DesktopCommandError> {
    let envelope = request.to_ui_command_envelope(client_version)?;
    session.send_ui_command(envelope)
}
