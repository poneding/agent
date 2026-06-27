use std::str::FromStr;

use agent_types::ApprovalRequestId;
use agent_ui_bridge::{UiApprovalDecision, UiApprovalRequest, UiEventPayload, UiJsonFrame};

use crate::TuiEffect;

const TUI_ALLOW_REASON: &str = "approved from TUI";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApprovalPanelState {
    pub pending: Vec<UiApprovalRequest>,
    pub focused_index: Option<usize>,
}

impl ApprovalPanelState {
    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        let UiJsonFrame::Event(event) = frame else {
            return;
        };

        match &event.payload {
            UiEventPayload::ApprovalRequested(request) => self.upsert_pending((**request).clone()),
            UiEventPayload::ApprovalResolved { request_id, .. } => self.remove_pending(request_id),
            _ => {}
        }
    }

    pub fn reduce(&mut self, action: ApprovalPanelAction) -> TuiEffect {
        match action {
            ApprovalPanelAction::FocusNext => {
                self.focus_next();
                TuiEffect::None
            }
            ApprovalPanelAction::FocusPrevious => {
                self.focus_previous();
                TuiEffect::None
            }
            ApprovalPanelAction::Allow => self.resolve_focused(
                UiApprovalDecision::Allow,
                TUI_ALLOW_REASON.to_string(),
                self.focused_request()
                    .and_then(|request| request.max_duration_seconds),
            ),
            ApprovalPanelAction::Deny { reason } => {
                let reason = reason.trim();
                if reason.is_empty() {
                    TuiEffect::LocalDiagnostic("deny requires a reason".to_string())
                } else {
                    self.resolve_focused(UiApprovalDecision::Deny, reason.to_string(), None)
                }
            }
            ApprovalPanelAction::Abort => match self.focused_request() {
                Some(request) => TuiEffect::ApprovalAborted(request.request_id.clone()),
                None => TuiEffect::LocalDiagnostic("no approval is focused".to_string()),
            },
        }
    }

    #[must_use]
    pub fn focused_request(&self) -> Option<&UiApprovalRequest> {
        self.focused_index.and_then(|index| self.pending.get(index))
    }

    fn upsert_pending(&mut self, request: UiApprovalRequest) {
        if let Some(existing) = self
            .pending
            .iter_mut()
            .find(|existing| existing.request_id == request.request_id)
        {
            *existing = request;
        } else {
            self.pending.push(request);
        }

        if self.focused_index.is_none() {
            self.focused_index = Some(0);
        }
        self.clamp_focus();
    }

    fn remove_pending(&mut self, request_id: &str) {
        self.pending
            .retain(|request| request.request_id != request_id);
        self.clamp_focus();
    }

    fn focus_next(&mut self) {
        if self.pending.is_empty() {
            self.focused_index = None;
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        self.focused_index = Some((current + 1).min(self.pending.len() - 1));
    }

    fn focus_previous(&mut self) {
        if self.pending.is_empty() {
            self.focused_index = None;
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        self.focused_index = Some(current.saturating_sub(1));
    }

    fn resolve_focused(
        &self,
        decision: UiApprovalDecision,
        reason: String,
        lease_duration_seconds: Option<u64>,
    ) -> TuiEffect {
        let Some(request) = self.focused_request() else {
            return TuiEffect::LocalDiagnostic("no approval is focused".to_string());
        };
        let Ok(request_id) = ApprovalRequestId::from_str(&request.request_id) else {
            return TuiEffect::LocalDiagnostic(format!(
                "invalid approval request id: {}",
                request.request_id
            ));
        };

        TuiEffect::ResolveApproval {
            request_id,
            decision,
            reason,
            lease_duration_seconds,
        }
    }

    fn clamp_focus(&mut self) {
        if self.pending.is_empty() {
            self.focused_index = None;
            return;
        }

        let index = self.focused_index.unwrap_or(0).min(self.pending.len() - 1);
        self.focused_index = Some(index);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalPanelAction {
    FocusNext,
    FocusPrevious,
    Allow,
    Deny { reason: String },
    Abort,
}
